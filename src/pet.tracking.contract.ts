import { petTrackingMood } from "./pet";

const tx = (zh: string, en: string) => `${zh}|${en}`;

export const activePetTrackingMoodContract: string = petTrackingMood(true, tx);
export const pausedPetTrackingMoodContract: string = petTrackingMood(false, tx);
