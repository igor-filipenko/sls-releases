import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function isRcVersion(version: string): boolean {
  return version.includes("Candidate");
}

export function isMilestoneVersion(version: string): boolean {
  return version.includes("Milestone");
}
