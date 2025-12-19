import { ref, watch } from 'vue';

export function useStorage<T>(key: string, defaultValue: T) {
  const data = ref<T>(defaultValue);

  const load = () => {
    try {
      const stored = localStorage.getItem(key);
      if (stored) {
        data.value = JSON.parse(stored);
      }
    } catch (error) {
      console.error('Failed to load from storage:', error);
    }
  };

  const save = () => {
    try {
      localStorage.setItem(key, JSON.stringify(data.value));
    } catch (error) {
      console.error('Failed to save to storage:', error);
    }
  };

  watch(data, save, { deep: true });
  load();

  return data;
}
