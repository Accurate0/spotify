import { createContext, use } from "react";

/** URL-backed filter state, shared by every stats page so their numbers agree. */
export type FilterState = {
  /** Account slug, or null for all accounts. */
  account: string | null;
  /** "all", "30d", "12m" or a calendar year like "2023". */
  range: string;
  /** Include podcasts and audiobooks, not just music. */
  podcasts: boolean;
  /** Count skips under 30 seconds as plays. */
  allPlays: boolean;
};

export type StatsFilter = {
  accounts: string[];
  from: string | null;
  to: string | null;
  kinds: ("TRACK" | "EPISODE" | "AUDIOBOOK")[];
  minMsPlayed: number;
  timeZone: string;
};

export type FilterContextValue = {
  state: FilterState;
  setState: (update: Partial<FilterState>) => void;
  accounts: readonly { readonly id: string; readonly slug: string; readonly displayName: string }[];
  /** Calendar years with any listening, newest first. */
  years: number[];
  filter: StatsFilter;
  /** A filter change is loading; pages keep their previous render, dimmed. */
  isPending: boolean;
};

export const FilterContext = createContext<FilterContextValue | null>(null);

export function useFilter(): FilterContextValue {
  const value = use(FilterContext);
  if (!value) throw new Error("useFilter outside FilterProvider");
  return value;
}
