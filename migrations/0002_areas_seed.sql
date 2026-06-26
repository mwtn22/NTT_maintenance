-- Minimal administrative area master for the fixture-covered prefectures.
-- Real deployment: populate from 国土数値情報行政区域データ (KSJ N03).
-- geom is NULL until PostGIS shapes are loaded from KSJ shapefiles.

INSERT INTO areas (municipality_code, pref_code, pref_name, municipality_name) VALUES
-- 北海道
('01101', '01', '北海道', '札幌市白石区'),
('01204', '01', '北海道', '旭川市'),
-- 宮城県
('04101', '04', '宮城県', '仙台市青葉区'),
-- 東京都
('13101', '13', '東京都', '千代田区'),
('13104', '13', '東京都', '新宿区'),
-- 神奈川県
('14100', '14', '神奈川県', '横浜市'),
-- 富山県
('16201', '16', '富山県', '富山市'),
-- 愛知県
('23106', '23', '愛知県', '名古屋市中区'),
-- 大阪府
('27100', '27', '大阪府', '大阪市'),
-- 兵庫県
('28100', '28', '兵庫県', '神戸市'),
-- 広島県
('34105', '34', '広島県', '広島市南区'),
-- 愛媛県
('38201', '38', '愛媛県', '松山市'),
-- 福岡県
('40130', '40', '福岡県', '福岡市'),
('40135', '40', '福岡県', '福岡市博多区'),
-- 沖縄県
('47201', '47', '沖縄県', '那覇市')
ON CONFLICT (municipality_code) DO NOTHING;
