export function HeroSkeleton() {
  return (
    <section className="relative w-full h-[70vh] min-h-[500px] max-h-[800px]">
      <div className="absolute inset-0 skeleton" />
      <div className="absolute inset-0 flex items-end pb-16 lg:pb-24">
        <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 w-full">
          <div className="max-w-2xl space-y-4">
            <div className="flex gap-3">
              <div className="skeleton w-12 h-5 rounded-md" />
              <div className="skeleton w-16 h-5 rounded-md" />
              <div className="skeleton w-14 h-5 rounded-md" />
            </div>
            <div className="skeleton w-3/4 h-10 lg:h-14 rounded-lg" />
            <div className="skeleton w-1/2 h-6 rounded-md" />
            <div className="space-y-2">
              <div className="skeleton w-full h-4 rounded" />
              <div className="skeleton w-5/6 h-4 rounded" />
              <div className="skeleton w-2/3 h-4 rounded" />
            </div>
            <div className="flex gap-3 pt-2">
              <div className="skeleton w-36 h-12 rounded-xl" />
              <div className="skeleton w-32 h-12 rounded-xl" />
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

export function CardSkeleton({ size = "md" }: { size?: "sm" | "md" | "lg" }) {
  const dims = {
    sm: { w: "w-[130px] sm:w-[150px]", h: "h-[195px] sm:h-[225px]" },
    md: { w: "w-[160px] sm:w-[180px] lg:w-[200px]", h: "h-[240px] sm:h-[270px] lg:h-[300px]" },
    lg: { w: "w-[200px] sm:w-[240px] lg:w-[280px]", h: "h-[300px] sm:h-[360px] lg:h-[420px]" },
  };

  return (
    <div className={`${dims[size].w} shrink-0`}>
      <div className={`${dims[size].h} skeleton rounded-xl`} />
    </div>
  );
}

export function RowSkeleton({ count = 6, size = "md" }: { count?: number; size?: "sm" | "md" | "lg" }) {
  return (
    <section className="relative py-4 lg:py-6">
      <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 mb-3 lg:mb-4">
        <div className="skeleton w-40 h-6 rounded-md" />
      </div>
      <div className="flex gap-2.5 lg:gap-3 px-4 sm:px-6 lg:px-8 overflow-hidden">
        {Array.from({ length: count }).map((_, i) => (
          <CardSkeleton key={i} size={size} />
        ))}
      </div>
    </section>
  );
}
