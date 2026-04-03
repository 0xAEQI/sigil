import { useRef, useEffect } from "react";

export function VerticalLines() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const frameRef = useRef(0);
  const timeRef = useRef(0);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d", { alpha: true });
    if (!ctx) return;

    let w = 0;
    let h = 0;
    let dpr = 1;

    function resize() {
      dpr = window.devicePixelRatio || 1;
      w = window.innerWidth;
      h = window.innerHeight;
      canvas!.width = w * dpr;
      canvas!.height = h * dpr;
      canvas!.style.width = w + "px";
      canvas!.style.height = h + "px";
      ctx!.setTransform(dpr, 0, 0, dpr, 0, 0);
    }

    const observer = new ResizeObserver(resize);
    observer.observe(document.documentElement);
    resize();

    function tick() {
      timeRef.current += 0.003;
      const t = timeRef.current;

      ctx!.clearRect(0, 0, w, h);

      const lineCount = 120;
      const centerX = w * (0.5 + Math.sin(t * 0.7) * 0.08);
      const spread = w * 0.48;

      for (let i = 0; i < lineCount; i++) {
        const normalized = i / (lineCount - 1);
        const fromCenter = normalized * 2 - 1;

        const compressed = Math.sign(fromCenter) * Math.pow(Math.abs(fromCenter), 0.35);
        const x = centerX + compressed * spread;

        const density = 1 - Math.abs(fromCenter);
        const breathe = 0.6 + Math.sin(t * 1.2 + normalized * 6) * 0.15;

        const baseAlpha = (0.03 + density * 0.06) * breathe;

        const hue = 220 + density * 30 + Math.sin(t + normalized * 4) * 10;
        const sat = 20 + density * 40;
        const light = 60 + density * 20;

        ctx!.beginPath();
        ctx!.moveTo(x, 0);
        ctx!.lineTo(x, h);
        ctx!.strokeStyle = `hsla(${hue}, ${sat}%, ${light}%, ${baseAlpha})`;
        ctx!.lineWidth = 0.5 + density * 0.5;
        ctx!.stroke();
      }

      frameRef.current = requestAnimationFrame(tick);
    }

    frameRef.current = requestAnimationFrame(tick);

    return () => {
      cancelAnimationFrame(frameRef.current);
      observer.disconnect();
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 2,
        pointerEvents: "none",
      }}
    />
  );
}
