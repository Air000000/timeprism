const PET_CHARACTER_STORAGE_KEY = "timeprism.pet.character";
const PET_CHARACTER_LEGACY_PRIMARY_SRC = "/pet-character.png";

export const PET_CHARACTER_PRIMARY_SRC = "/慕沛灵Q版桌宠形象.png";
export const PET_CHARACTER_DEFAULT_SRC = "/pet-character-default.svg";
export const PET_CHARACTER_DOCKED_LEFT_SRC = "/左侧.png";
export const PET_CHARACTER_DOCKED_RIGHT_SRC = "/右侧.png";

export function resolvePetCharacterSrc(saved: string | null | undefined): string {
  const normalized = saved?.trim();
  if (!normalized) {
    return PET_CHARACTER_PRIMARY_SRC;
  }
  if (normalized === PET_CHARACTER_LEGACY_PRIMARY_SRC) {
    return PET_CHARACTER_PRIMARY_SRC;
  }
  return normalized;
}

export function getPetCharacterSrc(): string {
  try {
    return resolvePetCharacterSrc(window.localStorage.getItem(PET_CHARACTER_STORAGE_KEY));
  } catch {
    return PET_CHARACTER_PRIMARY_SRC;
  }
}

export function savePetCharacterSrc(src: string) {
  const normalized = src.trim();
  try {
    if (!normalized) {
      window.localStorage.removeItem(PET_CHARACTER_STORAGE_KEY);
    } else {
      window.localStorage.setItem(PET_CHARACTER_STORAGE_KEY, normalized);
    }
  } catch {
    // Ignore storage failures and still let the current session try to render.
  }
}
