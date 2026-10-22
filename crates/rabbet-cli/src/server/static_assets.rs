//! Embedded HTML, CSS, and JavaScript for the Maintainer Admin Console.

/// Returns the embedded Single-Page Application HTML for `GET /admin`.
pub fn get_admin_dashboard_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Rabbet — Foundry Maintainer Console</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <script>
    tailwind.config = {
      darkMode: 'class',
      theme: { extend: { colors: { darkbg: '#0f172a', panel: '#1e293b' } } }
    }
  </script>
</head>
<body class="bg-darkbg text-slate-100 min-h-screen font-sans antialiased p-6">
  <div class="max-w-6xl mx-auto space-y-6">
    <header class="flex justify-between items-center border-b border-slate-700 pb-4">
      <div>
        <h1 class="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
          <span>🐰 Rabbet</span>
          <span class="text-xs font-mono uppercase bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 px-2 py-0.5 rounded">Foundry</span>
        </h1>
        <p class="text-sm text-slate-400 mt-1">Compiler-Verified Telemetry Corpus & Open-Weight Training Admin</p>
      </div>
      <div class="flex items-center gap-3">
        <input id="tokenInput" type="password" placeholder="RABBET_ADMIN_TOKEN" class="bg-slate-800 border border-slate-700 text-xs rounded px-3 py-1.5 focus:outline-none focus:border-emerald-500 text-slate-200">
        <button onclick="fetchStats()" class="bg-emerald-600 hover:bg-emerald-500 text-xs font-medium px-3 py-1.5 rounded transition">Refresh</button>
      </div>
    </header>

    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <div class="bg-panel border border-slate-700/60 p-4 rounded-lg">
        <div class="text-xs text-slate-400 font-mono">TOTAL TRACES</div>
        <div id="statTotal" class="text-2xl font-bold mt-1 text-white">-</div>
      </div>
      <div class="bg-panel border border-slate-700/60 p-4 rounded-lg">
        <div class="text-xs text-slate-400 font-mono">GOLD DPO PAIRS</div>
        <div id="statGold" class="text-2xl font-bold mt-1 text-amber-400">-</div>
      </div>
      <div class="bg-panel border border-slate-700/60 p-4 rounded-lg">
        <div class="text-xs text-slate-400 font-mono">SYNTHETIC (OSS)</div>
        <div id="statSynthetic" class="text-2xl font-bold mt-1 text-blue-400">-</div>
      </div>
      <div class="bg-panel border border-slate-700/60 p-4 rounded-lg">
        <div class="text-xs text-slate-400 font-mono">DATABASE SIZE</div>
        <div id="statDbSize" class="text-2xl font-bold mt-1 text-slate-300">-</div>
      </div>
    </div>

    <div class="bg-panel border border-slate-700/60 p-6 rounded-lg space-y-4">
      <div class="flex justify-between items-center">
        <h2 class="text-lg font-semibold text-white">Parquet Dataset Compaction</h2>
        <button onclick="triggerCompact()" class="bg-indigo-600 hover:bg-indigo-500 text-xs font-semibold px-4 py-2 rounded transition">Compact to Parquet</button>
      </div>
      <p class="text-sm text-slate-400">Compiles master SQLite records into canonical <code class="text-indigo-400">router_sft.parquet</code>, <code class="text-indigo-400">hole_filler_sft.parquet</code>, and <code class="text-indigo-400">hole_filler_dpo.parquet</code> shards.</p>
      <div id="compactStatus" class="hidden text-xs font-mono p-3 bg-slate-900 border border-slate-800 rounded text-slate-300"></div>
    </div>

    <div class="bg-panel border border-slate-700/60 p-6 rounded-lg space-y-4">
      <h2 class="text-lg font-semibold text-white">Discovered Dado Pattern Candidates</h2>
      <p class="text-sm text-slate-400">Recurring AST skeletons mined from Gold Oracle fallbacks awaiting approval into the pattern catalogue.</p>
      <div id="candidatesList" class="space-y-3"><div class="text-xs text-slate-500 italic">No pending pattern candidates discovered yet.</div></div>
    </div>
  </div>

  <script>
    function getHeaders() {
      const token = document.getElementById('tokenInput').value;
      return token ? { 'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json' } : { 'Content-Type': 'application/json' };
    }
    async function fetchStats() {
      try {
        const res = await fetch('/api/v1/admin/stats', { headers: getHeaders() });
        if (!res.ok) throw new Error('Unauthorized or error: ' + res.status);
        const data = await res.json();
        document.getElementById('statTotal').innerText = data.total_traces.toLocaleString();
        document.getElementById('statGold').innerText = (data.dpo_pairs_count || 0).toLocaleString();
        document.getElementById('statSynthetic').innerText = (data.synthetic_count || 0).toLocaleString();
        document.getElementById('statDbSize').innerText = (data.db_size_bytes / (1024 * 1024)).toFixed(2) + ' MB';
      } catch (e) { console.error(e); }
    }
    async function triggerCompact() {
      const el = document.getElementById('compactStatus');
      el.classList.remove('hidden');
      el.innerText = 'Compacting Parquet shards...';
      try {
        const res = await fetch('/api/v1/admin/compact', { method: 'POST', headers: getHeaders() });
        const data = await res.json();
        el.innerText = 'Compaction completed! Shards: Router=' + data.router_sft_rows + ' rows, Filler=' + data.hole_filler_sft_rows + ' rows, DPO=' + data.hole_filler_dpo_rows + ' rows';
      } catch (e) { el.innerText = 'Error compacting: ' + e.message; }
    }
    window.onload = fetchStats;
  </script>
</body>
</html>"#
}

