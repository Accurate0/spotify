import { useMemo, useTransition, type ReactNode } from "react";
import { graphql, useLazyLoadQuery } from "react-relay";
import { useSearchParams } from "react-router";
import type { FilterProviderAccountsQuery } from "./__generated__/FilterProviderAccountsQuery.graphql";
import { FilterContext, type FilterContextValue, type FilterState } from "@/lib/filter";

const TIME_ZONE = Intl.DateTimeFormat().resolvedOptions().timeZone;

/** [from, to) for a range, as local midnights in the browser's time zone. */
function rangeBounds(range: string): { from: Date | null; to: Date | null } {
  const now = new Date();
  const midnight = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate());
  if (range === "30d") {
    const from = midnight(now);
    from.setDate(from.getDate() - 30);
    return { from, to: null };
  }
  if (range === "12m") {
    const from = midnight(now);
    from.setFullYear(from.getFullYear() - 1);
    return { from, to: null };
  }
  const year = Number(range);
  if (Number.isInteger(year) && year > 1900) {
    return { from: new Date(year, 0, 1), to: new Date(year + 1, 0, 1) };
  }
  return { from: null, to: null };
}

/** Resolves URL filter state into the API filter and shares it with every stats page. */
export default function FilterProvider({ children }: { children: ReactNode }) {
  const data = useLazyLoadQuery<FilterProviderAccountsQuery>(
    graphql`
      query FilterProviderAccountsQuery {
        accounts {
          id
          slug
          displayName
        }
        stats(filter: { kinds: [TRACK, EPISODE, AUDIOBOOK], minMsPlayed: 0 }) {
          summary {
            firstPlayed
            lastPlayed
          }
        }
      }
    `,
    {},
  );

  const [params, setParams] = useSearchParams();
  const [isPending, startTransition] = useTransition();

  const value = useMemo<FilterContextValue>(() => {
    const state: FilterState = {
      account: params.get("account"),
      range: params.get("range") ?? "all",
      podcasts: params.get("podcasts") === "1",
      allPlays: params.get("plays") === "all",
    };

    const setState = (update: Partial<FilterState>) => {
      const next = { ...state, ...update };
      startTransition(() =>
        setParams(
        (current) => {
          const out = new URLSearchParams(current);
          const put = (key: string, value: string | null) =>
            value === null ? out.delete(key) : out.set(key, value);
          put("account", next.account);
          put("range", next.range === "all" ? null : next.range);
          put("podcasts", next.podcasts ? "1" : null);
          put("plays", next.allPlays ? "all" : null);
          return out;
        },
        { replace: true },
        ),
      );
    };

    const { firstPlayed, lastPlayed } = data.stats.summary;
    const years: number[] = [];
    if (firstPlayed && lastPlayed) {
      for (let y = new Date(lastPlayed).getFullYear(); y >= new Date(firstPlayed).getFullYear(); y--) {
        years.push(y);
      }
    }

    const account = data.accounts.find((a) => a.slug === state.account);
    const { from, to } = rangeBounds(state.range);

    return {
      state,
      setState,
      accounts: data.accounts,
      years,
      filter: {
        accounts: account ? [account.id] : [],
        from: from?.toISOString() ?? null,
        to: to?.toISOString() ?? null,
        kinds: state.podcasts ? ["TRACK", "EPISODE", "AUDIOBOOK"] : ["TRACK"],
        minMsPlayed: state.allPlays ? 0 : 30_000,
        timeZone: TIME_ZONE,
      },
      isPending,
    };
  }, [data, params, setParams, isPending]);

  return <FilterContext value={value}>{children}</FilterContext>;
}

