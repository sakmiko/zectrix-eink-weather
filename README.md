# ZECTRIX NOTE4 墨水屏气象站（V3 地区精化 & 免 Key 多源网页聚合版）

## 一、 V3 核心升级总览

1. **移除墨水屏顶栏数据来源标签（画面更纯粹）**：
   * 遵照需求移除了顶栏的信源胶囊（如 `[彩云 1KM]`、`[墨迹 PRO]`、`[中央台]`），顶栏左侧专注以 **`16×16` 大号点阵字** 呈现 **`[气象站] 北京 · 海淀区`** / **`上海 · 徐汇区`** / **`哈尔滨 · 南岗区`**，右侧呈现气象预警（或空气质量胶囊）与更新时间戳。

2. **细化到区县/商圈与经纬度的三级地址解析 + 独立地址簿 `weather_locations.json`**：
   * **结构化快速地址簿 (`weather_locations.json`)**：维护了覆盖全国 **36+ 核心城市、88 个重点区县/县级市** 的完整地址表（同时内嵌于二进制作为兜底，并支持外部 JSON 热加载与扩展），每条记录包含：
     * `province` / `city` / `district`（如 `北京市 / 北京 / 海淀区`、`江苏省 / 苏州 / 昆山市`）
     * `lon` / `lat`（区县级精确经纬度坐标）
     * `nmc_station_id` 与 `nmc_path`（[中央气象台](http://www.nmc.cn/)区县观测站号与网页路径，如海淀 `54399`、徐汇 `58367`）
     * `weather_cn_code`（[中国天气网](https://www.weather.com.cn/) 9 位区县代码，如海淀 `101010200`、浦东 `101020600`、南山 `101280604`）
     * `moji_slug`（[墨迹天气网页版](https://tianqi.moji.com/)区县路径，如 `beijing/haidian-district`）
     * `aliases`（高频商圈与地标别名，如 `"中关村"`、`"五道口"`、`"国贸"`、`"望京"`、`"陆家嘴"`、`"张江"`、`"科技园"`、`"前海"`、`"光谷"` 等）
   * **三种地区选择方式无缝切换**：
     1. **直接写城市+具体区域**：`WEATHER_LOCATION="北京海淀"`、`"北京市海淀区"`、`"上海·徐汇"`、`"深圳南山区"`、`"杭州余杭"`。
     2. **直接写区县名或地标商圈**：`WEATHER_LOCATION="海淀区"`、`"南山区"`、`"昆山"`、`"中关村"`、`"陆家嘴"`。
     3. **直接传经纬度坐标**：`WEATHER_LOCATION="116.2980,39.9599"`（自动识别并纠正经纬度顺序）或设置 `WEATHER_LON` + `WEATHER_LAT`。系统通过 **Haversine 球面大圆距离算法** 自动匹配距离最近的区县站点（获取其区县名与网页代码），同时保留用户输入的精确经纬度用于 1KM 网格气象查询。

3. **默认免 Key 多源聚合引擎 + 天气网页深度解析（`WEATHER_PROVIDER=aggregate`）**：
   * 默认无需配置任何商业 API Key，启动即并发拉取并融合以下 **4 大免 Key 数据源（含 3 个国内权威天气网页解析器 + 1 个全球高精度经纬度网格源）**：

| 免 Key 数据源 | 接入方式与解析目标 | 核心提取字段 | 优势互补作用 |
| :--- | :--- | :--- | :--- |
| **[中央气象台直连 + 网页解析](http://www.nmc.cn/)** (`nmc`) | 1. JSON 接口 `http://www.nmc.cn/rest/weather?stationid={id}`<br>2. 网页兜底 `http://www.nmc.cn/publish/forecast/{nmc_path}.html` | 国家级区县站实况温湿度、体感、风向风级、气压、国标 AQI、官方气象预警、7 日高低温 | 国家级基准气象站实测权威度最高，预警信号最准 |
| **[中国天气网网页解析](https://www.weather.com.cn/)** (`weathercn`) | 1. 预报页 `http://www.weather.com.cn/weather1d/{code}.shtml` 内嵌 `var hour3data` 与 `hidden_title`<br>2. 实况脚本 `http://d1.weather.com.cn/sk_2d/{code}.html` (`var dataSK`) | 区县级实时温度、相对湿度、风向风级、气压、AQI、未来 24H 逐 3 小时温度曲线、7 日天气与温差 | 覆盖全国全部 2800+ 区县，网页内嵌 JSON 脚本结构稳定 |
| **[墨迹天气网页版 HTML 解析](https://tianqi.moji.com/)** (`moji_web`) | 解析 `https://tianqi.moji.com/weather/china/{moji_slug}` DOM 节点 (`div.wea_weather`、`div.wea_about`、`div.wea_alert`、`ul.days`) | 实时温度、`<b>` 精准天气现象、湿度、风级、AQI 胶囊、预警胶囊及未来多日预报行 | 国内本土化天气描述最贴近日常习惯，区县级页面免鉴权直出 |
| **[Open-Meteo 经纬度网格源](https://open-meteo.com/)** (`open_meteo`) | `GET https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=...&minutely_15=precipitation&hourly=...&daily=...` | 任意经纬度实时温度、体感、湿度、气压、紫外线、日落时间、**未来两小时 15 分钟级降水雷达柱**、24H 逐时温度 | 零 Key 门槛支持全球任意经纬度坐标，补齐免 Key 源的短临分钟级降水雷达柱与逐时曲线 |

   * **多源中位数共识融合算法 (`aggregate_weather_snapshots`)**：
     * **抗异常值中位数融合**：对多源返回的实时温度 (`temp_c`)、体感温度 (`feels_like`)、湿度 (`humidity`)、气压 (`pressure`)、空气质量 (`aqi`)、逐时曲线 (`hourly_points`) 及未来 4 日高低温分别计算**中位数（Median Consensus）**，彻底消除单一网页改版、缓存延迟或局部站点异常带来的跳变。
     * **优势字段智能拼装**：优先采用国内官方气象源（中央台/中国天气网/墨迹网页）的本土化中文天气现象描述与气象灾害预警，同时融合 Open-Meteo（或彩云/和风）的高精度分钟级降水雷达柱 (`minute_bars`) 与逐时温度走势。

## 二、 快速地址查询与切换命令

1. **检索地址簿（支持省/市/区县/地标商圈/站号模糊搜索）**：
   ```bash
   cargo run --bin zectrix_weather_companion -- --list-locations 海淀
   cargo run --bin zectrix_weather_companion -- --list-locations 陆家嘴
   cargo run --bin zectrix_weather_companion -- --list-locations 深圳
   ```
2. **命令行免 Key 多源聚合秒级查天气（支持区县名、地标或经纬度）**：
   ```bash
   cargo run --bin zectrix_weather_companion -- --query "北京·海淀区"
   cargo run --bin zectrix_weather_companion -- --query "上海徐汇"
   cargo run --bin zectrix_weather_companion -- --query "114.0579,22.5431"
   ```
3. **运行时 HTTP 接口动态检索与切换地区（立即重绘上屏）**：
   * 检索地址簿：`curl "http://127.0.0.1:9597/locations?q=南山"`
   * 按城市区县切换：`curl "http://127.0.0.1:9597/location?q=深圳南山区"`
   * 按经纬度切换：`curl "http://127.0.0.1:9597/location?q=120.1551,30.2741"`
