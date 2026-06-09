import { invokeCommand } from "../client";
import type { Category } from "../types";

export async function listCategories(): Promise<Category[]> {
  return invokeCommand("list_categories");
}

export async function createCategory(input: {
  parent_id: number;
  name: string;
  color_hex?: string;
}): Promise<number> {
  return invokeCommand("create_category", { input });
}
