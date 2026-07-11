import { motion } from "motion/react";
import { useEffect, useId, useState } from "react";

export function AnimatedBeam({
  containerRef,
  fromRef,
  toRef,
  duration = 5,
  delay = 0,
  pathColor = "#aab5ae",
  gradientStartColor = "#1c9f70",
  gradientStopColor = "#75dbab",
}) {
  const id = useId();
  const [path, setPath] = useState("");
  const [size, setSize] = useState({ width: 0, height: 0 });

  useEffect(() => {
    const update = () => {
      if (!containerRef.current || !fromRef.current || !toRef.current) return;
      const root = containerRef.current.getBoundingClientRect();
      const from = fromRef.current.getBoundingClientRect();
      const to = toRef.current.getBoundingClientRect();
      const x1 = from.left - root.left + from.width / 2;
      const y1 = from.top - root.top + from.height / 2;
      const x2 = to.left - root.left + to.width / 2;
      const y2 = to.top - root.top + to.height / 2;
      setSize({ width: root.width, height: root.height });
      setPath(`M ${x1},${y1} Q ${(x1 + x2) / 2},${(y1 + y2) / 2} ${x2},${y2}`);
    };
    const observer = new ResizeObserver(update);
    if (containerRef.current) observer.observe(containerRef.current);
    update();
    return () => observer.disconnect();
  }, [containerRef, fromRef, toRef]);

  return (
    <svg className="animated-beam" width={size.width} height={size.height} viewBox={`0 0 ${size.width} ${size.height}`} aria-hidden="true">
      <path d={path} stroke={pathColor} strokeWidth="1" strokeOpacity="0.32" fill="none" />
      <path d={path} stroke={`url(#${id})`} strokeWidth="2" fill="none" />
      <defs>
        <motion.linearGradient
          id={id}
          gradientUnits="userSpaceOnUse"
          initial={{ x1: "0%", x2: "0%" }}
          animate={{ x1: ["-20%", "100%"], x2: ["0%", "120%"] }}
          transition={{ duration, delay, repeat: Infinity, ease: "linear" }}
        >
          <stop stopColor={gradientStartColor} stopOpacity="0" />
          <stop offset="40%" stopColor={gradientStartColor} />
          <stop offset="60%" stopColor={gradientStopColor} />
          <stop offset="100%" stopColor={gradientStopColor} stopOpacity="0" />
        </motion.linearGradient>
      </defs>
    </svg>
  );
}
