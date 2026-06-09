import { ref } from "vue";

export function useErrorMessage() {
  const error = ref("");

  function setErrorMessage(e: unknown) {
    error.value = `${e}`;
  }

  return {
    error,
    setErrorMessage,
  };
}
