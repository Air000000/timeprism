import { summonPetWindow } from "../api";

export const summonPetWindowContract = summonPetWindow satisfies () => Promise<void>;
