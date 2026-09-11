import { ref } from "vue";

export function useErrorMessage() {
  const error = ref("");

  function setErrorMessage(e: unknown) {
    error.value = `${e}`;
  }

  function clearErrorMessage(expected?: unknown) {
    if (expected === undefined || error.value === `${expected}`) {
      error.value = "";
    }
  }

  return {
    error,
    setErrorMessage,
    clearErrorMessage,
  };
}
