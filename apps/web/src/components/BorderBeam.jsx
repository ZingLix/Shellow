import { motion } from "motion/react";

export function BorderBeam({
  size = 72,
  duration = 7,
  delay = 0,
  colorFrom = "#1c9f70",
  colorTo = "#75dbab",
}) {
  return (
    <div className="magic-border-beam" aria-hidden="true">
      <motion.div
        className="magic-border-beam-light"
        style={{
          width: size,
          background: `linear-gradient(90deg, transparent, ${colorFrom}, ${colorTo}, transparent)`,
          offsetPath: `rect(0 auto auto 0 round ${size}px)`,
        }}
        initial={{ offsetDistance: "0%" }}
        animate={{ offsetDistance: "100%" }}
        transition={{ repeat: Infinity, ease: "linear", duration, delay: -delay }}
      />
    </div>
  );
}
