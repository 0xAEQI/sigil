import { useEffect, useRef, useCallback } from "react";
import * as THREE from "three";

const PARTICLE_COUNT = 1400;
const RETURN_FORCE = 0.04;
const DAMPING = 0.88;
const MOUSE_RADIUS = 60;
const MOUSE_FORCE = 8;

export default function ParticleLogo({
  size = 300,
  onReady,
}: {
  size?: number;
  onReady?: () => void;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const mouseRef = useRef({ x: 9999, y: 9999 });

  const handleMouseMove = useCallback((e: MouseEvent) => {
    const el = containerRef.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    mouseRef.current.x = e.clientX - rect.left - rect.width / 2;
    mouseRef.current.y = -(e.clientY - rect.top - rect.height / 2);
  }, []);

  const handleMouseLeave = useCallback(() => {
    mouseRef.current.x = 9999;
    mouseRef.current.y = 9999;
  }, []);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const scene = new THREE.Scene();
    const camera = new THREE.OrthographicCamera(
      -size / 2, size / 2, size / 2, -size / 2, 1, 1000
    );
    camera.position.z = 100;

    const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
    renderer.setSize(size, size);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    container.appendChild(renderer.domElement);

    // State arrays
    const positions = new Float32Array(PARTICLE_COUNT * 3);
    const targets = new Float32Array(PARTICLE_COUNT * 3);
    const velocities = new Float32Array(PARTICLE_COUNT * 3);
    const sizes = new Float32Array(PARTICLE_COUNT);
    const opacities = new Float32Array(PARTICLE_COUNT);

    // Init — will be placed on glyph after sampling
    for (let i = 0; i < PARTICLE_COUNT; i++) {
      positions[i * 3] = 0;
      positions[i * 3 + 1] = 0;
      positions[i * 3 + 2] = 0;
      velocities[i * 3] = 0;
      velocities[i * 3 + 1] = 0;
      velocities[i * 3 + 2] = 0;
      sizes[i] = 1.5 + Math.random() * 2.0;
      opacities[i] = 0.2 + Math.random() * 0.6;
    }

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    geometry.setAttribute("aSize", new THREE.BufferAttribute(sizes, 1));
    geometry.setAttribute("aOpacity", new THREE.BufferAttribute(opacities, 1));

    const material = new THREE.ShaderMaterial({
      transparent: true,
      depthWrite: false,
      uniforms: {
        uPixelRatio: { value: renderer.getPixelRatio() },
      },
      vertexShader: `
        attribute float aSize;
        attribute float aOpacity;
        varying float vOpacity;
        uniform float uPixelRatio;
        void main() {
          vOpacity = aOpacity;
          vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
          gl_PointSize = aSize * uPixelRatio;
          gl_Position = projectionMatrix * mvPosition;
        }
      `,
      fragmentShader: `
        varying float vOpacity;
        void main() {
          float d = length(gl_PointCoord - vec2(0.5));
          if (d > 0.5) discard;
          float alpha = smoothstep(0.5, 0.1, d) * vOpacity;
          gl_FragColor = vec4(0.0, 0.0, 0.0, alpha * 0.7);
        }
      `,
    });

    const points = new THREE.Points(geometry, material);
    scene.add(points);

    // Sample glyph
    function sampleGlyph(): Float32Array {
      const canvas = document.createElement("canvas");
      const s = size;
      canvas.width = s;
      canvas.height = s;
      const ctx = canvas.getContext("2d")!;
      ctx.fillStyle = "#000";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.font = `bold ${s * 0.78}px Inter, system-ui, sans-serif`;
      ctx.fillText("æ", s / 2, s / 2 + s * 0.03);

      const imageData = ctx.getImageData(0, 0, s, s);
      const filled: [number, number][] = [];
      for (let y = 0; y < s; y += 1) {
        for (let x = 0; x < s; x += 1) {
          if (imageData.data[(y * s + x) * 4 + 3] > 128) {
            filled.push([x - s / 2, -(y - s / 2)]);
          }
        }
      }

      const result = new Float32Array(PARTICLE_COUNT * 3);
      for (let i = 0; i < PARTICLE_COUNT; i++) {
        const pt = filled[Math.floor(Math.random() * filled.length)];
        result[i * 3] = pt[0] + (Math.random() - 0.5) * 2;
        result[i * 3 + 1] = pt[1] + (Math.random() - 0.5) * 2;
        result[i * 3 + 2] = 0;
      }
      return result;
    }

    const glyphTargets = sampleGlyph();
    // Start particles ON the glyph (solid æ)
    for (let i = 0; i < PARTICLE_COUNT * 3; i++) {
      targets[i] = glyphTargets[i];
      positions[i] = glyphTargets[i];
    }
    (geometry.attributes.position as THREE.BufferAttribute).needsUpdate = true;

    // After a brief pause, burst particles outward
    let hasBurst = false;
    const BURST_FRAME = 50; // ~0.8s at 60fps
    const BURST_FORCE = 12;

    let frame = 0;
    let animId: number;

    // Mouse events
    container.addEventListener("mousemove", handleMouseMove);
    container.addEventListener("mouseleave", handleMouseLeave);
    window.addEventListener("mousemove", handleMouseMove);

    function animate() {
      animId = requestAnimationFrame(animate);
      frame++;
      const time = frame * 0.008;

      // Before burst: render static, skip all physics
      if (!hasBurst) {
        if (frame === BURST_FRAME) {
          hasBurst = true;
          for (let i = 0; i < PARTICLE_COUNT; i++) {
            const angle = Math.random() * Math.PI * 2;
            const force = BURST_FORCE * (0.5 + Math.random());
            velocities[i * 3] = Math.cos(angle) * force;
            velocities[i * 3 + 1] = Math.sin(angle) * force;
          }
        } else {
          renderer.render(scene, camera);
          return;
        }
      }

      const mx = mouseRef.current.x;
      const my = mouseRef.current.y;
      const pos = geometry.attributes.position as THREE.BufferAttribute;

      for (let i = 0; i < PARTICLE_COUNT; i++) {
        const i3 = i * 3;

        // Breathing noise kicks in after reform
        const breathe = frame > BURST_FRAME + 60;
        const nx = breathe ? Math.sin(time * 0.6 + i * 0.07) * 0.15 : 0;
        const ny = breathe ? Math.cos(time * 0.5 + i * 0.09) * 0.15 : 0;

        // Spring to target
        const dx = targets[i3] - positions[i3];
        const dy = targets[i3 + 1] - positions[i3 + 1];

        velocities[i3] += dx * RETURN_FORCE + nx;
        velocities[i3 + 1] += dy * RETURN_FORCE + ny;

        // Mouse repulsion
        const mdx = positions[i3] - mx;
        const mdy = positions[i3 + 1] - my;
        const mDist = Math.sqrt(mdx * mdx + mdy * mdy);
        if (mDist < MOUSE_RADIUS && mDist > 0.1) {
          const force = (1 - mDist / MOUSE_RADIUS) * MOUSE_FORCE;
          velocities[i3] += (mdx / mDist) * force;
          velocities[i3 + 1] += (mdy / mDist) * force;
        }

        velocities[i3] *= DAMPING;
        velocities[i3 + 1] *= DAMPING;

        positions[i3] += velocities[i3];
        positions[i3 + 1] += velocities[i3 + 1];
      }


      pos.needsUpdate = true;
      renderer.render(scene, camera);
    }

    animate();

    return () => {
      cancelAnimationFrame(animId);
      container.removeEventListener("mousemove", handleMouseMove);
      container.removeEventListener("mouseleave", handleMouseLeave);
      window.removeEventListener("mousemove", handleMouseMove);
      renderer.dispose();
      geometry.dispose();
      material.dispose();
      if (container.contains(renderer.domElement)) {
        container.removeChild(renderer.domElement);
      }
    };
  }, [size, onReady, handleMouseMove, handleMouseLeave]);

  return <div ref={containerRef} className="inline-block cursor-none" />;
}
