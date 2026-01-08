//! NAPI bindings for Vue compiler.

use glob::glob;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rayon::prelude::*;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use vize_allocator::Bump;

use crate::{CompileResult, CompilerOptions};
use vize_compiler_core::{
    codegen::generate,
    options::{CodegenMode, CodegenOptions, TransformOptions},
    parser::parse,
    transform::transform,
};
use vize_compiler_vapor::{compile_vapor as vapor_compile, VaporCompilerOptions};

/// Compile Vue template to VDom render function
#[napi]
pub fn compile(template: String, options: Option<CompilerOptions>) -> Result<CompileResult> {
    let opts = options.unwrap_or_default();
    let allocator = Bump::new();

    // Parse
    let (mut root, errors) = parse(&allocator, &template);

    if !errors.is_empty() {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Parse errors: {:?}", errors),
        ));
    }

    // Determine mode
    let is_module_mode = opts.mode.as_deref() == Some("module");

    // Transform
    // In module mode, prefix_identifiers defaults to true (like Vue)
    let transform_opts = TransformOptions {
        prefix_identifiers: opts.prefix_identifiers.unwrap_or(is_module_mode),
        hoist_static: opts.hoist_static.unwrap_or(false),
        cache_handlers: opts.cache_handlers.unwrap_or(false),
        scope_id: opts.scope_id.clone().map(|s| s.into()),
        ssr: opts.ssr.unwrap_or(false),
        ..Default::default()
    };
    transform(&allocator, &mut root, transform_opts);

    // Codegen
    let codegen_opts = CodegenOptions {
        mode: if is_module_mode {
            CodegenMode::Module
        } else {
            CodegenMode::Function
        },
        source_map: opts.source_map.unwrap_or(false),
        ssr: opts.ssr.unwrap_or(false),
        ..Default::default()
    };
    let result = generate(&root, codegen_opts);

    // Collect helpers
    let helpers: Vec<String> = root.helpers.iter().map(|h| h.name().to_string()).collect();

    // Build AST JSON
    let ast = build_ast_json(&root);

    Ok(CompileResult {
        code: result.code.to_string(),
        preamble: result.preamble.to_string(),
        ast,
        map: None,
        helpers,
        templates: None,
    })
}

/// Compile Vue template to Vapor mode
#[napi(js_name = "compileVapor")]
pub fn compile_vapor(template: String, options: Option<CompilerOptions>) -> Result<CompileResult> {
    let opts = options.unwrap_or_default();
    let allocator = Bump::new();

    // Use actual Vapor compiler
    let vapor_opts = VaporCompilerOptions {
        prefix_identifiers: opts.prefix_identifiers.unwrap_or(false),
        ssr: opts.ssr.unwrap_or(false),
        ..Default::default()
    };
    let result = vapor_compile(&allocator, &template, vapor_opts);

    if !result.error_messages.is_empty() {
        return Err(Error::new(
            Status::GenericFailure,
            result.error_messages.join("\n"),
        ));
    }

    Ok(CompileResult {
        code: result.code,
        preamble: String::new(),
        ast: serde_json::json!({}),
        map: None,
        helpers: vec![],
        templates: Some(result.templates.iter().map(|s| s.to_string()).collect()),
    })
}

/// Parse template to AST only
#[napi]
pub fn parse_template(
    template: String,
    _options: Option<CompilerOptions>,
) -> Result<serde_json::Value> {
    let allocator = Bump::new();

    let (root, errors) = parse(&allocator, &template);

    if !errors.is_empty() {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Parse errors: {:?}", errors),
        ));
    }

    Ok(build_ast_json(&root))
}

/// SFC parse options for NAPI
#[napi(object)]
#[derive(Default)]
pub struct SfcParseOptionsNapi {
    pub filename: Option<String>,
}

/// SFC compile options for NAPI
#[napi(object)]
#[derive(Default)]
pub struct SfcCompileOptionsNapi {
    pub filename: Option<String>,
    pub source_map: Option<bool>,
    pub ssr: Option<bool>,
}

/// SFC compile result for NAPI
#[napi(object)]
pub struct SfcCompileResultNapi {
    /// Generated JavaScript code
    pub code: String,
    /// Generated CSS (if any)
    pub css: Option<String>,
    /// Compilation errors
    pub errors: Vec<String>,
    /// Compilation warnings
    pub warnings: Vec<String>,
}

/// Parse SFC (.vue file) - returns lightweight result for speed
#[napi(js_name = "parseSfc")]
pub fn parse_sfc(env: Env, source: String, options: Option<SfcParseOptionsNapi>) -> Result<Object> {
    use vize_compiler_sfc::{parse_sfc as sfc_parse, SfcParseOptions};

    let opts = options.unwrap_or_default();
    let parse_opts = SfcParseOptions {
        filename: opts.filename.unwrap_or_else(|| "anonymous.vue".to_string()),
        ..Default::default()
    };

    match sfc_parse(&source, parse_opts) {
        Ok(descriptor) => {
            // Build JS object directly for speed (avoid JSON serialization)
            let mut obj = env.create_object()?;

            obj.set("filename", descriptor.filename.as_ref())?;
            obj.set("source", descriptor.source.as_ref())?;

            // Template
            if let Some(ref template) = descriptor.template {
                let mut tpl_obj = env.create_object()?;
                tpl_obj.set("content", template.content.as_ref())?;
                tpl_obj.set("lang", template.lang.as_deref())?;
                obj.set("template", tpl_obj)?;
            } else {
                obj.set("template", env.get_null()?)?;
            }

            // Script
            if let Some(ref script) = descriptor.script {
                let mut scr_obj = env.create_object()?;
                scr_obj.set("content", script.content.as_ref())?;
                scr_obj.set("lang", script.lang.as_deref())?;
                scr_obj.set("setup", script.setup)?;
                obj.set("script", scr_obj)?;
            } else {
                obj.set("script", env.get_null()?)?;
            }

            // Script Setup
            if let Some(ref script_setup) = descriptor.script_setup {
                let mut scr_obj = env.create_object()?;
                scr_obj.set("content", script_setup.content.as_ref())?;
                scr_obj.set("lang", script_setup.lang.as_deref())?;
                scr_obj.set("setup", script_setup.setup)?;
                obj.set("scriptSetup", scr_obj)?;
            } else {
                obj.set("scriptSetup", env.get_null()?)?;
            }

            // Styles
            let mut styles_arr = env.create_array(descriptor.styles.len() as u32)?;
            for (i, style) in descriptor.styles.iter().enumerate() {
                let mut style_obj = env.create_object()?;
                style_obj.set("content", style.content.as_ref())?;
                style_obj.set("lang", style.lang.as_deref())?;
                style_obj.set("scoped", style.scoped)?;
                style_obj.set("module", style.module.as_deref())?;
                styles_arr.set(i as u32, style_obj)?;
            }
            obj.set("styles", styles_arr)?;

            // Custom blocks
            let mut customs_arr = env.create_array(descriptor.custom_blocks.len() as u32)?;
            for (i, block) in descriptor.custom_blocks.iter().enumerate() {
                let mut block_obj = env.create_object()?;
                block_obj.set("type", block.block_type.as_ref())?;
                block_obj.set("content", block.content.as_ref())?;
                customs_arr.set(i as u32, block_obj)?;
            }
            obj.set("customBlocks", customs_arr)?;

            Ok(obj)
        }
        Err(e) => Err(Error::new(Status::GenericFailure, e.message)),
    }
}

/// Compile SFC (.vue file) to JavaScript - main use case
#[napi(js_name = "compileSfc")]
pub fn compile_sfc(
    source: String,
    options: Option<SfcCompileOptionsNapi>,
) -> Result<SfcCompileResultNapi> {
    use vize_compiler_sfc::{
        compile_sfc as sfc_compile, parse_sfc as sfc_parse, ScriptCompileOptions,
        SfcCompileOptions, SfcParseOptions, StyleCompileOptions, TemplateCompileOptions,
    };

    let opts = options.unwrap_or_default();
    let filename = opts.filename.unwrap_or_else(|| "anonymous.vue".to_string());

    // Parse
    let parse_opts = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };

    let descriptor = match sfc_parse(&source, parse_opts) {
        Ok(d) => d,
        Err(e) => {
            return Ok(SfcCompileResultNapi {
                code: String::new(),
                css: None,
                errors: vec![e.message],
                warnings: vec![],
            });
        }
    };

    // Compile
    let has_scoped = descriptor.styles.iter().any(|s| s.scoped);
    let compile_opts = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(filename.clone()),
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename.clone()),
            scoped: has_scoped,
            ssr: opts.ssr.unwrap_or(false),
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename,
            scoped: has_scoped,
            ..Default::default()
        },
    };

    match sfc_compile(&descriptor, compile_opts) {
        Ok(result) => Ok(SfcCompileResultNapi {
            code: result.code,
            css: result.css,
            errors: result.errors.into_iter().map(|e| e.message).collect(),
            warnings: result.warnings.into_iter().map(|e| e.message).collect(),
        }),
        Err(e) => Ok(SfcCompileResultNapi {
            code: String::new(),
            css: None,
            errors: vec![e.message],
            warnings: vec![],
        }),
    }
}

/// Batch compile options for NAPI
#[napi(object)]
#[derive(Default)]
pub struct BatchCompileOptionsNapi {
    pub ssr: Option<bool>,
    pub threads: Option<u32>,
}

/// Batch compile result for NAPI
#[napi(object)]
pub struct BatchCompileResultNapi {
    /// Number of files compiled successfully
    pub success: u32,
    /// Number of files that failed
    pub failed: u32,
    /// Total input bytes
    pub input_bytes: u32,
    /// Total output bytes
    pub output_bytes: u32,
    /// Compilation time in milliseconds
    pub time_ms: f64,
}

/// Batch compile SFC files matching a glob pattern (native multithreading)
#[napi(js_name = "compileSfcBatch")]
pub fn compile_sfc_batch(
    pattern: String,
    options: Option<BatchCompileOptionsNapi>,
) -> Result<BatchCompileResultNapi> {
    use std::time::Instant;
    use vize_compiler_sfc::{
        compile_sfc as sfc_compile, parse_sfc as sfc_parse, ScriptCompileOptions,
        SfcCompileOptions, SfcParseOptions, StyleCompileOptions, TemplateCompileOptions,
    };

    let opts = options.unwrap_or_default();
    let ssr = opts.ssr.unwrap_or(false);

    // Configure thread pool if specified
    if let Some(threads) = opts.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads as usize)
            .build_global()
            .ok(); // Ignore if already configured
    }

    // Collect files matching the pattern
    let files: Vec<_> = glob(&pattern)
        .map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Invalid glob pattern: {}", e),
            )
        })?
        .filter_map(|entry| entry.ok())
        .filter(|path| path.extension().is_some_and(|ext| ext == "vue"))
        .collect();

    if files.is_empty() {
        return Err(Error::new(
            Status::GenericFailure,
            "No .vue files found matching the pattern",
        ));
    }

    let success = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    let input_bytes = AtomicUsize::new(0);
    let output_bytes = AtomicUsize::new(0);

    let start = Instant::now();

    // Compile files in parallel using rayon
    files.par_iter().for_each(|path| {
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                failed.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };

        input_bytes.fetch_add(source.len(), Ordering::Relaxed);

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("anonymous.vue")
            .to_string();

        // Parse
        let parse_opts = SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        };

        let descriptor = match sfc_parse(&source, parse_opts) {
            Ok(d) => d,
            Err(_) => {
                failed.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };

        // Compile
        let has_scoped = descriptor.styles.iter().any(|s| s.scoped);
        let compile_opts = SfcCompileOptions {
            parse: SfcParseOptions {
                filename: filename.clone(),
                ..Default::default()
            },
            script: ScriptCompileOptions {
                id: Some(filename.clone()),
                ..Default::default()
            },
            template: TemplateCompileOptions {
                id: Some(filename.clone()),
                scoped: has_scoped,
                ssr,
                ..Default::default()
            },
            style: StyleCompileOptions {
                id: filename,
                scoped: has_scoped,
                ..Default::default()
            },
        };

        match sfc_compile(&descriptor, compile_opts) {
            Ok(result) => {
                success.fetch_add(1, Ordering::Relaxed);
                output_bytes.fetch_add(result.code.len(), Ordering::Relaxed);
            }
            Err(_) => {
                failed.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    let elapsed = start.elapsed();

    Ok(BatchCompileResultNapi {
        success: success.load(Ordering::Relaxed) as u32,
        failed: failed.load(Ordering::Relaxed) as u32,
        input_bytes: input_bytes.load(Ordering::Relaxed) as u32,
        output_bytes: output_bytes.load(Ordering::Relaxed) as u32,
        time_ms: elapsed.as_secs_f64() * 1000.0,
    })
}

/// Build AST JSON from root node
fn build_ast_json(root: &vize_compiler_core::RootNode<'_>) -> serde_json::Value {
    use vize_compiler_core::TemplateChildNode;

    let children: Vec<serde_json::Value> = root
        .children
        .iter()
        .map(|child| match child {
            TemplateChildNode::Element(el) => serde_json::json!({
                "type": "ELEMENT",
                "tag": el.tag.as_str(),
                "tagType": format!("{:?}", el.tag_type),
                "props": el.props.len(),
                "children": el.children.len(),
                "isSelfClosing": el.is_self_closing,
            }),
            TemplateChildNode::Text(text) => serde_json::json!({
                "type": "TEXT",
                "content": text.content.as_str(),
            }),
            TemplateChildNode::Comment(comment) => serde_json::json!({
                "type": "COMMENT",
                "content": comment.content.as_str(),
            }),
            TemplateChildNode::Interpolation(interp) => serde_json::json!({
                "type": "INTERPOLATION",
                "content": match &interp.content {
                    vize_compiler_core::ExpressionNode::Simple(exp) => exp.content.as_str(),
                    _ => "<compound>",
                }
            }),
            _ => serde_json::json!({
                "type": "UNKNOWN"
            }),
        })
        .collect();

    serde_json::json!({
        "type": "ROOT",
        "children": children,
        "helpers": root.helpers.iter().map(|h| h.name()).collect::<Vec<_>>(),
        "components": root.components.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
        "directives": root.directives.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
    })
}
