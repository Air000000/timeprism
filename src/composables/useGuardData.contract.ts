import { ref } from "vue";
import { useGuardData } from "./useGuardData";

const autoCaptureEnabled = ref(false);

const guard = useGuardData({
  tx: (_zh, en) => en,
  mappedTypeText: (mappedType) => mappedType,
  refreshData: async () => {},
  setErrorMessage: (_error: unknown) => {},
  autoCaptureEnabled,
  persistAutoCaptureEnabled: async (_enabled: boolean) => {},
});

export const guardTrackingContract = guard satisfies {
  autoCaptureEnabled: Readonly<typeof autoCaptureEnabled>;
  onAutoCaptureToggle: (event: Event) => Promise<void>;
};
