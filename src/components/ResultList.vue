<script setup lang="ts">
import type { SearchResult } from "../types";
import ResultItem from "./ResultItem.vue";

defineProps<{
  results: SearchResult[];
  selectedIndex: number;
}>();

defineEmits<{
  action: [result: SearchResult];
}>();
</script>

<template>
  <div
    v-if="results.length > 0"
    class="flex min-h-0 flex-1 flex-col overflow-y-auto py-2"
    role="listbox"
    aria-label="Search results"
  >
    <ResultItem
      v-for="(result, idx) in results"
      :key="result.id"
      :result="result"
      :selected="idx === selectedIndex"
      role="option"
      :aria-selected="idx === selectedIndex"
      @action="$emit('action', result)"
    />
  </div>
</template>
