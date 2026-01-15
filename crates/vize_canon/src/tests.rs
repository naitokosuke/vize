//! Snapshot tests for vize_canon.

#[cfg(test)]
mod virtual_ts_tests {
    use crate::sfc_typecheck::{type_check_sfc, SfcTypeCheckOptions};

    /// Generate virtual TypeScript from SFC using canon's type_check_sfc.
    /// This uses croquis scope analysis to generate proper JavaScript scoping
    /// (for-of loops, closures, IIFEs) instead of declare statements.
    fn generate_virtual_ts_from_sfc(source: &str) -> String {
        let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
        let result = type_check_sfc(source, &options);
        result.virtual_ts.unwrap_or_default()
    }

    #[test]
    fn snapshot_virtual_ts_simple_component() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'

const count = ref(0)
const message = ref('Hello')

function increment() {
  count.value++
}
</script>

<template>
  <div>
    <p>{{ message }}</p>
    <p>Count: {{ count }}</p>
    <button @click="increment">+1</button>
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_simple_component", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_with_props() {
        let source = r#"<script setup lang="ts">
interface Props {
  title: string
  count?: number
}

const props = defineProps<Props>()
</script>

<template>
  <h1>{{ props.title }}</h1>
  <p v-if="props.count">Count: {{ props.count }}</p>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_with_props", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_with_emits() {
        let source = r#"<script setup lang="ts">
interface Emits {
  (e: 'update', value: number): void
  (e: 'close'): void
}

const emit = defineEmits<Emits>()

function handleClick() {
  emit('update', 42)
}
</script>

<template>
  <button @click="handleClick">Update</button>
  <button @click="emit('close')">Close</button>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_with_emits", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_with_v_for() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'

const items = ref([1, 2, 3])
</script>

<template>
  <ul>
    <li v-for="(item, index) in items" :key="index">
      {{ index }}: {{ item }}
    </li>
  </ul>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_with_v_for", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_with_slots() {
        let source = r#"<script setup lang="ts">
import { useSlots } from 'vue'

const slots = useSlots()
</script>

<template>
  <div>
    <slot name="header" :title="'Header'"></slot>
    <slot></slot>
    <slot name="footer"></slot>
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_with_slots", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_complex_component() {
        let source = r#"<script setup lang="ts">
import { ref, computed, watch } from 'vue'

interface Props {
  initialCount?: number
  title: string
}

interface Emits {
  (e: 'change', value: number): void
}

const props = withDefaults(defineProps<Props>(), {
  initialCount: 0
})

const emit = defineEmits<Emits>()

const count = ref(props.initialCount)
const doubled = computed(() => count.value * 2)

function increment() {
  count.value++
  emit('change', count.value)
}

watch(count, (newVal) => {
  console.log('Count changed:', newVal)
})
</script>

<template>
  <div class="counter">
    <h1>{{ props.title }}</h1>
    <p>Count: {{ count }}</p>
    <p>Doubled: {{ doubled }}</p>
    <button @click="increment">+1</button>
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_complex_component", virtual_ts);
    }

    #[test]
    fn snapshot_virtual_ts_with_composables() {
        let source = r#"<script setup lang="ts">
import { useMouse } from '@vueuse/core'

const { x, y } = useMouse()
</script>

<template>
  <div>
    Mouse position: {{ x }}, {{ y }}
  </div>
</template>"#;

        let virtual_ts = generate_virtual_ts_from_sfc(source);
        insta::assert_snapshot!("virtual_ts_with_composables", virtual_ts);
    }
}
