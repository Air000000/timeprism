import type { Ref } from "vue";
import { useErrorMessage } from "./useErrorMessage";
import { useHomeData } from "./useHomeData";

const errorApi = useErrorMessage();

export const errorOwnershipContract = errorApi satisfies {
  error: Ref<string>;
  setErrorMessage: (error: unknown) => void;
  clearErrorMessage: (expected?: unknown) => void;
};

type HomeDataOptions = Parameters<typeof useHomeData>[0];

export const homeErrorOwnershipContract = {
  clearErrorMessage: (_expected?: unknown) => undefined,
} satisfies Pick<HomeDataOptions, "clearErrorMessage">;
