"use strict";

// Approximate prefecture centroids (lat, lng), keyed by JIS X 0401 code.
// Used to place map markers until per-event PostGIS geometry is populated.
const PREF_CENTROIDS = {
  "01": [43.2203, 142.8635, "北海道"], "02": [40.6907, 140.7402, "青森県"],
  "03": [39.6371, 141.2196, "岩手県"], "04": [38.4457, 140.8694, "宮城県"],
  "05": [39.5728, 140.3206, "秋田県"], "06": [38.4395, 140.1038, "山形県"],
  "07": [37.4399, 140.3596, "福島県"], "08": [36.3418, 140.4468, "茨城県"],
  "09": [36.6657, 139.8836, "栃木県"], "10": [36.4910, 139.0608, "群馬県"],
  "11": [36.0258, 139.3286, "埼玉県"], "12": [35.4072, 140.2496, "千葉県"],
  "13": [35.6895, 139.6917, "東京都"], "14": [35.4660, 139.3622, "神奈川県"],
  "15": [37.5611, 138.9536, "新潟県"], "16": [36.6953, 137.2113, "富山県"],
  "17": [36.5611, 136.6566, "石川県"], "18": [35.8838, 136.2241, "福井県"],
  "19": [35.6418, 138.5557, "山梨県"], "20": [36.1543, 138.0333, "長野県"],
  "21": [35.7779, 137.0596, "岐阜県"], "22": [34.9447, 138.3094, "静岡県"],
  "23": [35.0844, 137.0000, "愛知県"], "24": [34.5176, 136.4855, "三重県"],
  "25": [35.2014, 136.1500, "滋賀県"], "26": [35.1106, 135.6800, "京都府"],
  "27": [34.6863, 135.5200, "大阪府"], "28": [34.9000, 134.9000, "兵庫県"],
  "29": [34.4256, 135.7686, "奈良県"], "30": [33.8000, 135.5000, "和歌山県"],
  "31": [35.4833, 133.6000, "鳥取県"], "32": [35.2000, 132.7000, "島根県"],
  "33": [34.9000, 133.7000, "岡山県"], "34": [34.5000, 132.7000, "広島県"],
  "35": [34.2000, 131.5000, "山口県"], "36": [33.9000, 134.3000, "徳島県"],
  "37": [34.2000, 133.9000, "香川県"], "38": [33.7500, 132.8000, "愛媛県"],
  "39": [33.4000, 133.3000, "高知県"], "40": [33.6064, 130.4181, "福岡県"],
  "41": [33.2000, 130.1000, "佐賀県"], "42": [32.9000, 129.9000, "長崎県"],
  "43": [32.8000, 130.8000, "熊本県"], "44": [33.2000, 131.5000, "大分県"],
  "45": [32.2000, 131.4000, "宮崎県"], "46": [31.5000, 130.6000, "鹿児島県"],
  "47": [26.2124, 127.6809, "沖縄県"],
};

const KIND_LABEL = {
  construction: "工事", maintenance: "メンテナンス", fault: "故障",
  outage: "停電", voltage_sag: "瞬時電圧低下",
};
const STATUS_LABEL = {
  planned: "予定", active: "発生中", resolved: "復旧",
  historical: "履歴", unknown: "不明",
};
const SCOPE_LABEL = {
  general: "一般", high_voltage: "高圧", special_high_voltage: "特別高圧",
};
const VIS_LABEL = {
  public: "一般公開", restricted: "契約者限定",
  excluded_from_public_outage_page: "停電面では非掲載",
};

let map, markerLayer;
const markersByEvent = {};
let mapReady = false;

function initMap() {
  // Degrade gracefully: if Leaflet failed to load (offline/CDN blocked),
  // keep the list usable and show a notice in the map pane.
  if (typeof L === "undefined") {
    const el = document.getElementById("map");
    el.innerHTML = "<p style='padding:1rem;color:#475467'>地図ライブラリを読み込めませんでした。"
      + "一覧は引き続きご利用いただけます。</p>";
    return;
  }
  map = L.map("map").setView([37.5, 137.0], 5);
  // 国土地理院 標準地図タイル（出典明示）。
  L.tileLayer("https://cyberjapandata.gsi.go.jp/xyz/std/{z}/{x}/{y}.png", {
    attribution: "<a href='https://maps.gsi.go.jp/development/ichiran.html'>国土地理院</a>",
    maxZoom: 18,
  }).addTo(map);
  markerLayer = L.layerGroup().addTo(map);
  mapReady = true;
}

function populatePrefSelect() {
  const sel = document.getElementById("f-pref");
  Object.keys(PREF_CENTROIDS).sort().forEach((code) => {
    const opt = document.createElement("option");
    opt.value = code;
    opt.textContent = PREF_CENTROIDS[code][2];
    sel.appendChild(opt);
  });
}

function buildQuery() {
  const params = new URLSearchParams();
  for (const id of ["family", "event_kind", "status", "pref_code", "q"]) {
    const el = document.querySelector(`[name="${id}"]`);
    if (el && el.value) params.set(id, el.value);
  }
  params.set("per_page", "200");
  return params.toString();
}

function fmtJst(iso) {
  if (!iso) return "—";
  const d = new Date(iso);
  return d.toLocaleString("ja-JP", { timeZone: "Asia/Tokyo",
    month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit" });
}

function badge(cls, text) {
  return `<span class="badge ${cls}">${text}</span>`;
}

function renderList(items) {
  const ul = document.getElementById("events");
  ul.innerHTML = "";
  for (const e of items) {
    const li = document.createElement("li");
    li.className = "event";
    li.tabIndex = 0;
    li.dataset.eventId = e.event_id;
    const scopeBadges = [];
    if (e.event_kind === "voltage_sag" || e.customer_scope !== "general") {
      scopeBadges.push(badge("scope", SCOPE_LABEL[e.customer_scope] || e.customer_scope));
    }
    if (e.visibility_status !== "public") {
      scopeBadges.push(badge("scope", VIS_LABEL[e.visibility_status] || e.visibility_status));
    }
    const areaNames = (e.areas || [])
      .map((a) => a.municipality_name || a.pref_name).filter(Boolean).join("、");
    const impact = e.impact && e.impact.value
      ? ` / ${e.impact.value.toLocaleString()}${e.impact.unit || ""}` : "";
    li.innerHTML = `
      <div class="badges">
        ${badge("family-" + e.family, e.family === "power" ? "電力" : "通信")}
        ${badge("kind", KIND_LABEL[e.event_kind] || e.event_kind)}
        ${badge("status-" + e.status, STATUS_LABEL[e.status] || e.status)}
        ${scopeBadges.join("")}
      </div>
      <div class="ev-title">${escapeHtml(e.title)}</div>
      <div class="ev-meta">${escapeHtml(e.provider.org_name)}${impact}
        ${areaNames ? " ・ " + escapeHtml(areaNames) : ""}</div>
      <div class="ev-times">
        発生: ${fmtJst(e.started_at)} ／ 復旧見込: ${fmtJst(e.recovery_estimate_at)}<br>
        公式更新: ${fmtJst(e.source_updated_at)}
        ${e.original_url ? ` ・ <a href="${escapeAttr(e.original_url)}" target="_blank" rel="noopener">原典 ↗</a>` : ""}
      </div>`;
    li.addEventListener("click", () => selectEvent(e.event_id));
    li.addEventListener("keydown", (ev) => {
      if (ev.key === "Enter" || ev.key === " ") { ev.preventDefault(); selectEvent(e.event_id); }
    });
    ul.appendChild(li);
  }
}

function renderMarkers(items) {
  if (!mapReady) return;
  markerLayer.clearLayers();
  for (const k of Object.keys(markersByEvent)) delete markersByEvent[k];
  // Group events by prefecture so co-located markers cluster naturally.
  for (const e of items) {
    const pref = (e.areas || []).find((a) => a.pref_code && PREF_CENTROIDS[a.pref_code]);
    if (!pref) continue;
    const [lat, lng, name] = PREF_CENTROIDS[pref.pref_code];
    // jitter slightly so multiple events in one pref don't fully overlap
    const j = () => (Math.random() - 0.5) * 0.18;
    const m = L.circleMarker([lat + j(), lng + j()], {
      radius: 8,
      color: e.family === "power" ? "#b54708" : "#175cd3",
      fillColor: e.status === "active" ? "#b42318" : "#667085",
      fillOpacity: 0.85, weight: 2,
    });
    m.bindPopup(`<strong>${escapeHtml(e.title)}</strong><br>${escapeHtml(e.provider.org_name)}<br>`
      + `${KIND_LABEL[e.event_kind] || e.event_kind} / ${STATUS_LABEL[e.status] || e.status} (${name})`);
    m.on("click", () => highlightListRow(e.event_id));
    m.addTo(markerLayer);
    markersByEvent[e.event_id] = m;
  }
}

function selectEvent(eventId) {
  highlightListRow(eventId);
  if (!mapReady) return;
  const m = markersByEvent[eventId];
  if (m) { map.setView(m.getLatLng(), 8); m.openPopup(); }
}

function highlightListRow(eventId) {
  document.querySelectorAll("li.event").forEach((li) => {
    li.classList.toggle("selected", li.dataset.eventId === eventId);
  });
  const row = document.querySelector(`li.event[data-event-id="${eventId}"]`);
  if (row) row.scrollIntoView({ block: "nearest", behavior: "smooth" });
}

function escapeHtml(s) {
  return String(s == null ? "" : s).replace(/[&<>"']/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
}
function escapeAttr(s) { return escapeHtml(s).replace(/"/g, "&quot;"); }

async function load() {
  const status = document.getElementById("status");
  status.textContent = "読み込み中…";
  try {
    const res = await fetch("/api/v1/events?" + buildQuery());
    if (!res.ok) throw new Error("HTTP " + res.status);
    const data = await res.json();
    renderList(data.items);
    renderMarkers(data.items);
    const now = new Date().toLocaleString("ja-JP", { timeZone: "Asia/Tokyo",
      hour: "2-digit", minute: "2-digit", second: "2-digit" });
    status.textContent = `${data.total} 件を表示（当サイト取得 ${now}）。`
      + `公式更新時刻は各行に表示します。`;
  } catch (err) {
    status.textContent = "データ取得に失敗しました: " + err.message;
  }
}

document.addEventListener("DOMContentLoaded", () => {
  initMap();
  populatePrefSelect();
  document.getElementById("filters").addEventListener("submit", (e) => {
    e.preventDefault(); load();
  });
  document.getElementById("clear").addEventListener("click", () => {
    setTimeout(load, 0); // let the form reset before reloading
  });
  load();
});
