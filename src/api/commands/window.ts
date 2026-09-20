import { invokeCommand } from "../client";

export function summonPetWindow(): Promise<void> {
  return invokeCommand<void>("summon_pet_window");
}
