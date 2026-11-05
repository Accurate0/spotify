import { useState } from "react";
import { Disc3, User } from "lucide-react";
import { cn } from "@/lib/utils";

const SIZES = {
  sm: "size-10",
  md: "size-16",
  lg: "size-40 sm:size-48",
} as const;

/**
 * Album or artist artwork served from /art (filled in by enrichment). Shows a quiet
 * placeholder until the image exists, or if it fails to load.
 */
export default function Artwork({
  url,
  alt,
  size = "sm",
  round = false,
}: {
  url?: string | null;
  alt: string;
  size?: keyof typeof SIZES;
  /** Artists are round, albums square. */
  round?: boolean;
}) {
  const [failed, setFailed] = useState(false);
  const shape = cn(SIZES[size], "shrink-0 overflow-hidden", round ? "rounded-full" : "rounded-md");

  if (!url || failed) {
    const Icon = round ? User : Disc3;
    return (
      <div className={cn(shape, "bg-muted text-muted-foreground grid place-items-center")} aria-hidden>
        <Icon className={size === "lg" ? "size-12" : "size-4"} strokeWidth={1.5} />
      </div>
    );
  }

  return (
    <img
      src={url}
      alt={alt}
      loading="lazy"
      decoding="async"
      onError={() => setFailed(true)}
      className={cn(shape, "bg-muted object-cover")}
    />
  );
}
