import { motion } from "framer-motion";

interface TreeNode {
  id: string;
  x: number;
  y: number;
  label: string;
  tier: number;
}

interface TreeEdge {
  from: string;
  to: string;
}

const nodes: TreeNode[] = [
  { id: "root", x: 50, y: 12, label: "root", tier: 0 },
  { id: "shadow", x: 28, y: 40, label: "shadow", tier: 1 },
  { id: "ops", x: 72, y: 40, label: "ops", tier: 1 },
  { id: "engineer", x: 10, y: 68, label: "engineer", tier: 2 },
  { id: "reviewer", x: 28, y: 68, label: "reviewer", tier: 2 },
  { id: "researcher", x: 46, y: 68, label: "researcher", tier: 2 },
  { id: "monitor", x: 64, y: 68, label: "monitor", tier: 2 },
  { id: "deployer", x: 84, y: 68, label: "deployer", tier: 2 },
];

const edges: TreeEdge[] = [
  { from: "root", to: "shadow" },
  { from: "root", to: "ops" },
  { from: "shadow", to: "engineer" },
  { from: "shadow", to: "reviewer" },
  { from: "shadow", to: "researcher" },
  { from: "ops", to: "monitor" },
  { from: "ops", to: "deployer" },
];

function getNode(id: string): TreeNode {
  return nodes.find((n) => n.id === id)!;
}

const nodeColors: Record<number, string> = {
  0: "#818cf8",
  1: "#818cf8",
  2: "rgba(129,140,248,0.55)",
};

const nodeRadii: Record<number, number> = {
  0: 2.8,
  1: 2,
  2: 1.5,
};

const fadeUp = (delay = 0) => ({
  initial: { opacity: 0, y: 24 } as const,
  whileInView: { opacity: 1, y: 0 } as const,
  viewport: { once: true, margin: "-60px" } as const,
  transition: { duration: 0.7, ease: "easeOut" as const, delay },
});

export function AgentTree() {
  return (
    <section className="relative z-10 py-28 px-8">
      <div className="max-w-4xl mx-auto">
        {/* Section header */}
        <motion.div {...fadeUp()} className="text-center mb-16">
          <p
            className="text-[11px] uppercase tracking-[0.25em] text-white/15 mb-5"
            style={{ fontFamily: "'Space Grotesk', sans-serif" }}
          >
            The Tree
          </p>
          <h2
            className="text-2xl md:text-3xl font-medium text-white/80 mb-4 leading-snug"
            style={{ fontFamily: "'Space Grotesk', sans-serif" }}
          >
            One primitive. Infinite structure.
          </h2>
          <p className="text-[15px] text-white/25 max-w-lg mx-auto leading-relaxed">
            Everything is an agent in a tree. Departments, teams, specialists
            — patterns that emerge from how agents arrange themselves.
          </p>
        </motion.div>

        {/* SVG Tree Visualization */}
        <motion.div
          {...fadeUp(0.15)}
          className="relative max-w-2xl mx-auto"
        >
          {/* Glow behind tree */}
          <div
            className="absolute inset-0 -inset-x-12 pointer-events-none"
            style={{
              background:
                "radial-gradient(ellipse 60% 50% at 50% 40%, rgba(99,102,241,0.06) 0%, transparent 70%)",
            }}
          />

          <svg
            viewBox="0 0 100 85"
            className="w-full"
            style={{ overflow: "visible" }}
          >
            <defs>
              <filter id="node-glow">
                <feGaussianBlur stdDeviation="1.5" result="blur" />
                <feMerge>
                  <feMergeNode in="blur" />
                  <feMergeNode in="SourceGraphic" />
                </feMerge>
              </filter>
              <filter id="node-glow-strong">
                <feGaussianBlur stdDeviation="2.5" result="blur" />
                <feMerge>
                  <feMergeNode in="blur" />
                  <feMergeNode in="SourceGraphic" />
                </feMerge>
              </filter>
            </defs>

            {/* Edges — draw in from parent to child */}
            {edges.map((edge, i) => {
              const from = getNode(edge.from);
              const to = getNode(edge.to);
              return (
                <motion.path
                  key={`edge-${i}`}
                  d={`M ${from.x} ${from.y} L ${to.x} ${to.y}`}
                  stroke="rgba(129,140,248,0.12)"
                  strokeWidth="0.4"
                  fill="none"
                  initial={{ pathLength: 0, opacity: 0 }}
                  whileInView={{ pathLength: 1, opacity: 1 }}
                  viewport={{ once: true, margin: "-60px" }}
                  transition={{
                    duration: 0.8,
                    ease: "easeOut",
                    delay: 0.3 + i * 0.08,
                  }}
                />
              );
            })}

            {/* Nodes */}
            {nodes.map((node) => (
              <motion.g
                key={node.id}
                initial={{ opacity: 0, scale: 0 }}
                whileInView={{ opacity: 1, scale: 1 }}
                viewport={{ once: true, margin: "-60px" }}
                transition={{
                  duration: 0.5,
                  ease: [0.16, 1, 0.3, 1],
                  delay: 0.15 + node.tier * 0.3,
                }}
                style={{ transformOrigin: `${node.x}px ${node.y}px` }}
              >
                {/* Outer glow ring for root */}
                {node.tier === 0 && (
                  <circle
                    cx={node.x}
                    cy={node.y}
                    r={5}
                    fill="none"
                    stroke="rgba(129,140,248,0.08)"
                    strokeWidth="0.3"
                    className="animate-pulse-slow"
                  />
                )}

                {/* Node circle */}
                <circle
                  cx={node.x}
                  cy={node.y}
                  r={nodeRadii[node.tier]}
                  fill={nodeColors[node.tier]}
                  filter={
                    node.tier === 0
                      ? "url(#node-glow-strong)"
                      : "url(#node-glow)"
                  }
                />

                {/* Label */}
                <text
                  x={node.x}
                  y={node.y + (node.tier === 0 ? 6.5 : 5.5)}
                  textAnchor="middle"
                  fill="rgba(255,255,255,0.3)"
                  fontSize="2.5"
                  fontFamily="'JetBrains Mono', monospace"
                  fontWeight="400"
                >
                  {node.label}
                </text>
              </motion.g>
            ))}
          </svg>

          {/* Caption */}
          <motion.p
            {...fadeUp(0.5)}
            className="text-center mt-10 text-[13px] text-white/15 leading-relaxed max-w-md mx-auto"
          >
            This tree wasn't designed. It was grown. The user said "help with code"
            and the root agent spawned an engineer. The same kernel handles one agent or thousands.
          </motion.p>
        </motion.div>
      </div>
    </section>
  );
}
