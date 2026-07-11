import { motion, useInView } from "motion/react";
import { useRef } from "react";

export function BlurFade({
  children,
  className = "",
  delay = 0,
  duration = 0.5,
  offset = 16,
  direction = "up",
}) {
  const ref = useRef(null);
  const inView = useInView(ref, { once: true, margin: "-72px" });
  const axis = direction === "left" || direction === "right" ? "x" : "y";
  const sign = direction === "left" || direction === "up" ? 1 : -1;

  return (
    <motion.div
      ref={ref}
      className={className}
      initial={{ opacity: 0, filter: "blur(5px)", [axis]: sign * offset }}
      animate={inView ? { opacity: 1, filter: "blur(0px)", [axis]: 0 } : undefined}
      transition={{ duration, delay, ease: "easeOut" }}
    >
      {children}
    </motion.div>
  );
}
