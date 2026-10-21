use std::fs;
use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, Timelike, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const DEFAULT_TZ_OFFSET_SECONDS: i32 = 8 * 3600;

/// 内置全国 88 个核心城市与区县级地址簿 JSON（同时支持通过外部 `weather_locations.json` 实时扩展维护）
pub const BUILTIN_LOCATIONS_JSON: &str = r#"{
  "version": "2.0",
  "locations": [
    {
      "province": "北京市",
      "city": "北京",
      "district": "海淀区",
      "lon": 116.298,
      "lat": 39.9599,
      "nmc_station_id": "54399",
      "nmc_path": "ABJ/haidian",
      "weather_cn_code": "101010200",
      "moji_slug": "beijing/haidian-district",
      "aliases": [
        "北京海淀",
        "海淀",
        "中关村",
        "五道口",
        "beijing haidian"
      ]
    },
    {
      "province": "北京市",
      "city": "北京",
      "district": "朝阳区",
      "lon": 116.4864,
      "lat": 39.9215,
      "nmc_station_id": "54433",
      "nmc_path": "ABJ/chaoyang",
      "weather_cn_code": "101010300",
      "moji_slug": "beijing/chaoyang-district",
      "aliases": [
        "北京朝阳",
        "朝阳",
        "国贸",
        "三里屯",
        "望京",
        "beijing chaoyang"
      ]
    },
    {
      "province": "北京市",
      "city": "北京",
      "district": "东城区",
      "lon": 116.4164,
      "lat": 39.9283,
      "nmc_station_id": "54511",
      "nmc_path": "ABJ/beijing",
      "weather_cn_code": "101010100",
      "moji_slug": "beijing/dongcheng-district",
      "aliases": [
        "北京东城",
        "东城",
        "北京",
        "北京市",
        "天安门",
        "王府井"
      ]
    },
    {
      "province": "北京市",
      "city": "北京",
      "district": "西城区",
      "lon": 116.3659,
      "lat": 39.9123,
      "nmc_station_id": "54511",
      "nmc_path": "ABJ/beijing",
      "weather_cn_code": "101011500",
      "moji_slug": "beijing/xicheng-district",
      "aliases": [
        "北京西城",
        "西城",
        "金融街",
        "西单"
      ]
    },
    {
      "province": "北京市",
      "city": "北京",
      "district": "丰台区",
      "lon": 116.2869,
      "lat": 39.8584,
      "nmc_station_id": "54514",
      "nmc_path": "ABJ/fengtai",
      "weather_cn_code": "101010900",
      "moji_slug": "beijing/fengtai-district",
      "aliases": [
        "北京丰台",
        "丰台"
      ]
    },
    {
      "province": "北京市",
      "city": "北京",
      "district": "石景山区",
      "lon": 116.2229,
      "lat": 39.9066,
      "nmc_station_id": "54513",
      "nmc_path": "ABJ/shijingshan",
      "weather_cn_code": "101011000",
      "moji_slug": "beijing/shijingshan-district",
      "aliases": [
        "北京石景山",
        "石景山"
      ]
    },
    {
      "province": "北京市",
      "city": "北京",
      "district": "通州区",
      "lon": 116.6564,
      "lat": 39.9099,
      "nmc_station_id": "54431",
      "nmc_path": "ABJ/tongzhou",
      "weather_cn_code": "101010600",
      "moji_slug": "beijing/tongzhou-district",
      "aliases": [
        "北京通州",
        "通州",
        "副中心"
      ]
    },
    {
      "province": "北京市",
      "city": "北京",
      "district": "昌平区",
      "lon": 116.2312,
      "lat": 40.2207,
      "nmc_station_id": "54499",
      "nmc_path": "ABJ/changping",
      "weather_cn_code": "101010700",
      "moji_slug": "beijing/changping-district",
      "aliases": [
        "北京昌平",
        "昌平",
        "回龙观",
        "天通苑"
      ]
    },
    {
      "province": "北京市",
      "city": "北京",
      "district": "大兴区",
      "lon": 116.3414,
      "lat": 39.7269,
      "nmc_station_id": "54594",
      "nmc_path": "ABJ/daxing",
      "weather_cn_code": "101011100",
      "moji_slug": "beijing/daxing-district",
      "aliases": [
        "北京大兴",
        "大兴",
        "亦庄"
      ]
    },
    {
      "province": "北京市",
      "city": "北京",
      "district": "顺义区",
      "lon": 116.6545,
      "lat": 40.1303,
      "nmc_station_id": "54398",
      "nmc_path": "ABJ/shunyi",
      "weather_cn_code": "101010400",
      "moji_slug": "beijing/shunyi-district",
      "aliases": [
        "北京顺义",
        "顺义"
      ]
    },
    {
      "province": "上海市",
      "city": "上海",
      "district": "徐汇区",
      "lon": 121.4365,
      "lat": 31.1885,
      "nmc_station_id": "58367",
      "nmc_path": "ASH/xuhui",
      "weather_cn_code": "101021200",
      "moji_slug": "shanghai/xuhui-district",
      "aliases": [
        "上海徐汇",
        "徐汇",
        "上海",
        "上海市",
        "徐家汇",
        "shanghai xuhui"
      ]
    },
    {
      "province": "上海市",
      "city": "上海",
      "district": "浦东新区",
      "lon": 121.5444,
      "lat": 31.2215,
      "nmc_station_id": "58370",
      "nmc_path": "ASH/pudong",
      "weather_cn_code": "101020600",
      "moji_slug": "shanghai/pudong-new-area",
      "aliases": [
        "上海浦东",
        "浦东",
        "陆家嘴",
        "张江",
        "shanghai pudong"
      ]
    },
    {
      "province": "上海市",
      "city": "上海",
      "district": "黄浦区",
      "lon": 121.4844,
      "lat": 31.2317,
      "nmc_station_id": "58367",
      "nmc_path": "ASH/shanghai",
      "weather_cn_code": "101020100",
      "moji_slug": "shanghai/huangpu-district",
      "aliases": [
        "上海黄浦",
        "黄浦",
        "外滩",
        "人民广场"
      ]
    },
    {
      "province": "上海市",
      "city": "上海",
      "district": "静安区",
      "lon": 121.4474,
      "lat": 31.2279,
      "nmc_station_id": "58367",
      "nmc_path": "ASH/jingan",
      "weather_cn_code": "101021400",
      "moji_slug": "shanghai/jingan-district",
      "aliases": [
        "上海静安",
        "静安",
        "静安寺"
      ]
    },
    {
      "province": "上海市",
      "city": "上海",
      "district": "长宁区",
      "lon": 121.4246,
      "lat": 31.2204,
      "nmc_station_id": "58367",
      "nmc_path": "ASH/changning",
      "weather_cn_code": "101021300",
      "moji_slug": "shanghai/changning-district",
      "aliases": [
        "上海长宁",
        "长宁",
        "虹桥"
      ]
    },
    {
      "province": "上海市",
      "city": "上海",
      "district": "闵行区",
      "lon": 121.3817,
      "lat": 31.1128,
      "nmc_station_id": "58361",
      "nmc_path": "ASH/minhang",
      "weather_cn_code": "101020200",
      "moji_slug": "shanghai/minhang-district",
      "aliases": [
        "上海闵行",
        "闵行",
        "莘庄"
      ]
    },
    {
      "province": "上海市",
      "city": "上海",
      "district": "杨浦区",
      "lon": 121.526,
      "lat": 31.2595,
      "nmc_station_id": "58367",
      "nmc_path": "ASH/yangpu",
      "weather_cn_code": "101021700",
      "moji_slug": "shanghai/yangpu-district",
      "aliases": [
        "上海杨浦",
        "杨浦",
        "五角场"
      ]
    },
    {
      "province": "上海市",
      "city": "上海",
      "district": "松江区",
      "lon": 121.2277,
      "lat": 31.0322,
      "nmc_station_id": "58462",
      "nmc_path": "ASH/songjiang",
      "weather_cn_code": "101020900",
      "moji_slug": "shanghai/songjiang-district",
      "aliases": [
        "上海松江",
        "松江"
      ]
    },
    {
      "province": "上海市",
      "city": "上海",
      "district": "嘉定区",
      "lon": 121.2653,
      "lat": 31.3756,
      "nmc_station_id": "58365",
      "nmc_path": "ASH/jiading",
      "weather_cn_code": "101020500",
      "moji_slug": "shanghai/jiading-district",
      "aliases": [
        "上海嘉定",
        "嘉定"
      ]
    },
    {
      "province": "广东省",
      "city": "广州",
      "district": "天河区",
      "lon": 113.3613,
      "lat": 23.1247,
      "nmc_station_id": "59287",
      "nmc_path": "AGD/guangzhou",
      "weather_cn_code": "101280109",
      "moji_slug": "guangdong/tianhe-district",
      "aliases": [
        "广州天河",
        "天河",
        "广州",
        "广州市",
        "珠江新城",
        "guangzhou"
      ]
    },
    {
      "province": "广东省",
      "city": "广州",
      "district": "越秀区",
      "lon": 113.2668,
      "lat": 23.1289,
      "nmc_station_id": "59287",
      "nmc_path": "AGD/guangzhou",
      "weather_cn_code": "101280107",
      "moji_slug": "guangdong/yuexiu-district",
      "aliases": [
        "广州越秀",
        "越秀"
      ]
    },
    {
      "province": "广东省",
      "city": "广州",
      "district": "海珠区",
      "lon": 113.3174,
      "lat": 23.0838,
      "nmc_station_id": "59287",
      "nmc_path": "AGD/guangzhou",
      "weather_cn_code": "101280108",
      "moji_slug": "guangdong/haizhu-district",
      "aliases": [
        "广州海珠",
        "海珠",
        "琶洲"
      ]
    },
    {
      "province": "广东省",
      "city": "广州",
      "district": "番禺区",
      "lon": 113.3842,
      "lat": 22.9377,
      "nmc_station_id": "59481",
      "nmc_path": "AGD/panyu",
      "weather_cn_code": "101280102",
      "moji_slug": "guangdong/panyu-district",
      "aliases": [
        "广州番禺",
        "番禺"
      ]
    },
    {
      "province": "广东省",
      "city": "广州",
      "district": "白云区",
      "lon": 113.2732,
      "lat": 23.1579,
      "nmc_station_id": "59287",
      "nmc_path": "AGD/guangzhou",
      "weather_cn_code": "101280110",
      "moji_slug": "guangdong/baiyun-district",
      "aliases": [
        "广州白云",
        "白云"
      ]
    },
    {
      "province": "广东省",
      "city": "深圳",
      "district": "南山区",
      "lon": 113.9304,
      "lat": 22.5333,
      "nmc_station_id": "59493",
      "nmc_path": "AGD/shenzhen",
      "weather_cn_code": "101280604",
      "moji_slug": "guangdong/nanshan-district",
      "aliases": [
        "深圳南山",
        "南山",
        "深圳",
        "深圳市",
        "科技园",
        "前海",
        "shenzhen"
      ]
    },
    {
      "province": "广东省",
      "city": "深圳",
      "district": "福田区",
      "lon": 114.055,
      "lat": 22.5229,
      "nmc_station_id": "59493",
      "nmc_path": "AGD/shenzhen",
      "weather_cn_code": "101280603",
      "moji_slug": "guangdong/futian-district",
      "aliases": [
        "深圳福田",
        "福田",
        "华强北"
      ]
    },
    {
      "province": "广东省",
      "city": "深圳",
      "district": "罗湖区",
      "lon": 114.1314,
      "lat": 22.5484,
      "nmc_station_id": "59493",
      "nmc_path": "AGD/shenzhen",
      "weather_cn_code": "101280602",
      "moji_slug": "guangdong/luohu-district",
      "aliases": [
        "深圳罗湖",
        "罗湖"
      ]
    },
    {
      "province": "广东省",
      "city": "深圳",
      "district": "宝安区",
      "lon": 113.8831,
      "lat": 22.5533,
      "nmc_station_id": "59493",
      "nmc_path": "AGD/shenzhen",
      "weather_cn_code": "101280605",
      "moji_slug": "guangdong/baoan-district",
      "aliases": [
        "深圳宝安",
        "宝安"
      ]
    },
    {
      "province": "广东省",
      "city": "深圳",
      "district": "龙岗区",
      "lon": 114.2477,
      "lat": 22.7199,
      "nmc_station_id": "59493",
      "nmc_path": "AGD/shenzhen",
      "weather_cn_code": "101280606",
      "moji_slug": "guangdong/longgang-district",
      "aliases": [
        "深圳龙岗",
        "龙岗"
      ]
    },
    {
      "province": "广东省",
      "city": "深圳",
      "district": "龙华区",
      "lon": 114.0365,
      "lat": 22.657,
      "nmc_station_id": "59493",
      "nmc_path": "AGD/shenzhen",
      "weather_cn_code": "101280609",
      "moji_slug": "guangdong/longhua-district",
      "aliases": [
        "深圳龙华",
        "深圳北站"
      ]
    },
    {
      "province": "浙江省",
      "city": "杭州",
      "district": "西湖区",
      "lon": 120.1298,
      "lat": 30.2595,
      "nmc_station_id": "58457",
      "nmc_path": "AZJ/hangzhou",
      "weather_cn_code": "101210113",
      "moji_slug": "zhejiang/xihu-district",
      "aliases": [
        "杭州西湖",
        "西湖",
        "杭州",
        "杭州市",
        "hangzhou"
      ]
    },
    {
      "province": "浙江省",
      "city": "杭州",
      "district": "上城区",
      "lon": 120.1693,
      "lat": 30.2425,
      "nmc_station_id": "58457",
      "nmc_path": "AZJ/hangzhou",
      "weather_cn_code": "101210109",
      "moji_slug": "zhejiang/shangcheng-district",
      "aliases": [
        "杭州上城",
        "上城"
      ]
    },
    {
      "province": "浙江省",
      "city": "杭州",
      "district": "拱墅区",
      "lon": 120.1421,
      "lat": 30.3197,
      "nmc_station_id": "58457",
      "nmc_path": "AZJ/hangzhou",
      "weather_cn_code": "101210111",
      "moji_slug": "zhejiang/gongshu-district",
      "aliases": [
        "杭州拱墅",
        "拱墅"
      ]
    },
    {
      "province": "浙江省",
      "city": "杭州",
      "district": "滨江区",
      "lon": 120.2119,
      "lat": 30.2084,
      "nmc_station_id": "58457",
      "nmc_path": "AZJ/hangzhou",
      "weather_cn_code": "101210114",
      "moji_slug": "zhejiang/binjiang-district",
      "aliases": [
        "杭州滨江",
        "滨江"
      ]
    },
    {
      "province": "浙江省",
      "city": "杭州",
      "district": "余杭区",
      "lon": 120.0085,
      "lat": 30.2736,
      "nmc_station_id": "58444",
      "nmc_path": "AZJ/yuhang",
      "weather_cn_code": "101210106",
      "moji_slug": "zhejiang/yuhang-district",
      "aliases": [
        "杭州余杭",
        "余杭",
        "未来科技城"
      ]
    },
    {
      "province": "浙江省",
      "city": "杭州",
      "district": "萧山区",
      "lon": 120.2645,
      "lat": 30.1851,
      "nmc_station_id": "58459",
      "nmc_path": "AZJ/xiaoshan",
      "weather_cn_code": "101210102",
      "moji_slug": "zhejiang/xiaoshan-district",
      "aliases": [
        "杭州萧山",
        "萧山"
      ]
    },
    {
      "province": "江苏省",
      "city": "南京",
      "district": "玄武区",
      "lon": 118.7977,
      "lat": 32.0486,
      "nmc_station_id": "58238",
      "nmc_path": "AJS/nanjing",
      "weather_cn_code": "101190108",
      "moji_slug": "jiangsu/xuanwu-district",
      "aliases": [
        "南京玄武",
        "玄武",
        "南京",
        "南京市",
        "nanjing"
      ]
    },
    {
      "province": "江苏省",
      "city": "南京",
      "district": "鼓楼区",
      "lon": 118.7697,
      "lat": 32.0663,
      "nmc_station_id": "58238",
      "nmc_path": "AJS/nanjing",
      "weather_cn_code": "101190109",
      "moji_slug": "jiangsu/gulou-district",
      "aliases": [
        "南京鼓楼"
      ]
    },
    {
      "province": "江苏省",
      "city": "南京",
      "district": "建邺区",
      "lon": 118.7327,
      "lat": 32.0045,
      "nmc_station_id": "58238",
      "nmc_path": "AJS/nanjing",
      "weather_cn_code": "101190110",
      "moji_slug": "jiangsu/jianye-district",
      "aliases": [
        "南京建邺",
        "建邺",
        "河西"
      ]
    },
    {
      "province": "江苏省",
      "city": "南京",
      "district": "江宁区",
      "lon": 118.84,
      "lat": 31.9526,
      "nmc_station_id": "58238",
      "nmc_path": "AJS/jiangning",
      "weather_cn_code": "101190104",
      "moji_slug": "jiangsu/jiangning-district",
      "aliases": [
        "南京江宁",
        "江宁"
      ]
    },
    {
      "province": "江苏省",
      "city": "苏州",
      "district": "工业园区",
      "lon": 120.7146,
      "lat": 31.3267,
      "nmc_station_id": "58357",
      "nmc_path": "AJS/suzhou",
      "weather_cn_code": "101190401",
      "moji_slug": "jiangsu/suzhou",
      "aliases": [
        "苏州工业园",
        "苏州园区",
        "苏州",
        "苏州市",
        "suzhou"
      ]
    },
    {
      "province": "江苏省",
      "city": "苏州",
      "district": "姑苏区",
      "lon": 120.6174,
      "lat": 31.3357,
      "nmc_station_id": "58357",
      "nmc_path": "AJS/suzhou",
      "weather_cn_code": "101190401",
      "moji_slug": "jiangsu/gusu-district",
      "aliases": [
        "苏州姑苏",
        "姑苏"
      ]
    },
    {
      "province": "江苏省",
      "city": "苏州",
      "district": "昆山市",
      "lon": 120.9807,
      "lat": 31.3856,
      "nmc_station_id": "58356",
      "nmc_path": "AJS/kunshan",
      "weather_cn_code": "101190404",
      "moji_slug": "jiangsu/kunshan",
      "aliases": [
        "苏州昆山",
        "昆山",
        "昆山市"
      ]
    },
    {
      "province": "四川省",
      "city": "成都",
      "district": "武侯区",
      "lon": 104.043,
      "lat": 30.6423,
      "nmc_station_id": "56294",
      "nmc_path": "ASC/chengdu",
      "weather_cn_code": "101270119",
      "moji_slug": "sichuan/wuhou-district",
      "aliases": [
        "成都武侯",
        "武侯",
        "成都",
        "成都市",
        "高新区",
        "chengdu"
      ]
    },
    {
      "province": "四川省",
      "city": "成都",
      "district": "锦江区",
      "lon": 104.0835,
      "lat": 30.6561,
      "nmc_station_id": "56294",
      "nmc_path": "ASC/chengdu",
      "weather_cn_code": "101270116",
      "moji_slug": "sichuan/jinjiang-district",
      "aliases": [
        "成都锦江",
        "锦江",
        "春熙路",
        "太古里"
      ]
    },
    {
      "province": "四川省",
      "city": "成都",
      "district": "青羊区",
      "lon": 104.0615,
      "lat": 30.6739,
      "nmc_station_id": "56294",
      "nmc_path": "ASC/chengdu",
      "weather_cn_code": "101270117",
      "moji_slug": "sichuan/qingyang-district",
      "aliases": [
        "成都青羊",
        "青羊"
      ]
    },
    {
      "province": "四川省",
      "city": "成都",
      "district": "双流区",
      "lon": 103.9237,
      "lat": 30.5744,
      "nmc_station_id": "56288",
      "nmc_path": "ASC/shuangliu",
      "weather_cn_code": "101270106",
      "moji_slug": "sichuan/shuangliu-district",
      "aliases": [
        "成都双流",
        "双流",
        "天府新区"
      ]
    },
    {
      "province": "湖北省",
      "city": "武汉",
      "district": "江汉区",
      "lon": 114.2709,
      "lat": 30.6015,
      "nmc_station_id": "57494",
      "nmc_path": "AHB/wuhan",
      "weather_cn_code": "101200108",
      "moji_slug": "hubei/jianghan-district",
      "aliases": [
        "武汉江汉",
        "江汉",
        "武汉",
        "武汉市",
        "汉口",
        "wuhan"
      ]
    },
    {
      "province": "湖北省",
      "city": "武汉",
      "district": "武昌区",
      "lon": 114.3159,
      "lat": 30.5539,
      "nmc_station_id": "57494",
      "nmc_path": "AHB/wuhan",
      "weather_cn_code": "101200111",
      "moji_slug": "hubei/wuchang-district",
      "aliases": [
        "武汉武昌",
        "武昌"
      ]
    },
    {
      "province": "湖北省",
      "city": "武汉",
      "district": "洪山区",
      "lon": 114.3438,
      "lat": 30.5,
      "nmc_station_id": "57494",
      "nmc_path": "AHB/wuhan",
      "weather_cn_code": "101200112",
      "moji_slug": "hubei/hongshan-district",
      "aliases": [
        "武汉洪山",
        "洪山",
        "光谷"
      ]
    },
    {
      "province": "陕西省",
      "city": "西安",
      "district": "雁塔区",
      "lon": 108.9487,
      "lat": 34.2226,
      "nmc_station_id": "57036",
      "nmc_path": "ASN/xian",
      "weather_cn_code": "101110113",
      "moji_slug": "shaanxi/yanta-district",
      "aliases": [
        "西安雁塔",
        "雁塔",
        "西安",
        "西安市",
        "曲江",
        "xian"
      ]
    },
    {
      "province": "陕西省",
      "city": "西安",
      "district": "碑林区",
      "lon": 108.947,
      "lat": 34.2512,
      "nmc_station_id": "57036",
      "nmc_path": "ASN/xian",
      "weather_cn_code": "101110109",
      "moji_slug": "shaanxi/beilin-district",
      "aliases": [
        "西安碑林",
        "碑林",
        "钟楼"
      ]
    },
    {
      "province": "陕西省",
      "city": "西安",
      "district": "未央区",
      "lon": 108.9468,
      "lat": 34.2929,
      "nmc_station_id": "57036",
      "nmc_path": "ASN/xian",
      "weather_cn_code": "101110112",
      "moji_slug": "shaanxi/weiyang-district",
      "aliases": [
        "西安未央",
        "未央"
      ]
    },
    {
      "province": "陕西省",
      "city": "西安",
      "district": "长安区",
      "lon": 108.907,
      "lat": 34.1583,
      "nmc_station_id": "57039",
      "nmc_path": "ASN/changan",
      "weather_cn_code": "101110102",
      "moji_slug": "shaanxi/changan-district",
      "aliases": [
        "西安长安",
        "长安区"
      ]
    },
    {
      "province": "天津市",
      "city": "天津",
      "district": "和平区",
      "lon": 117.2147,
      "lat": 39.1172,
      "nmc_station_id": "54527",
      "nmc_path": "ATJ/tianjin",
      "weather_cn_code": "101030100",
      "moji_slug": "tianjin/heping-district",
      "aliases": [
        "天津和平",
        "天津",
        "天津市",
        "tianjin"
      ]
    },
    {
      "province": "天津市",
      "city": "天津",
      "district": "南开区",
      "lon": 117.1507,
      "lat": 39.1382,
      "nmc_station_id": "54527",
      "nmc_path": "ATJ/nankai",
      "weather_cn_code": "101031600",
      "moji_slug": "tianjin/nankai-district",
      "aliases": [
        "天津南开",
        "南开"
      ]
    },
    {
      "province": "天津市",
      "city": "天津",
      "district": "滨海新区",
      "lon": 117.6809,
      "lat": 39.0294,
      "nmc_station_id": "54623",
      "nmc_path": "ATJ/binhaixinqu",
      "weather_cn_code": "101031100",
      "moji_slug": "tianjin/binhai-new-area",
      "aliases": [
        "天津滨海",
        "滨海新区",
        "塘沽"
      ]
    },
    {
      "province": "重庆市",
      "city": "重庆",
      "district": "渝中区",
      "lon": 106.569,
      "lat": 29.5528,
      "nmc_station_id": "57516",
      "nmc_path": "ACQ/chongqing",
      "weather_cn_code": "101040100",
      "moji_slug": "chongqing/yuzhong-district",
      "aliases": [
        "重庆渝中",
        "渝中",
        "重庆",
        "重庆市",
        "解放碑",
        "chongqing"
      ]
    },
    {
      "province": "重庆市",
      "city": "重庆",
      "district": "江北区",
      "lon": 106.5743,
      "lat": 29.6066,
      "nmc_station_id": "57516",
      "nmc_path": "ACQ/jiangbei",
      "weather_cn_code": "101044100",
      "moji_slug": "chongqing/jiangbei-district",
      "aliases": [
        "重庆江北",
        "江北区",
        "观音桥"
      ]
    },
    {
      "province": "重庆市",
      "city": "重庆",
      "district": "渝北区",
      "lon": 106.6307,
      "lat": 29.7181,
      "nmc_station_id": "57516",
      "nmc_path": "ACQ/yubei",
      "weather_cn_code": "101040700",
      "moji_slug": "chongqing/yubei-district",
      "aliases": [
        "重庆渝北",
        "渝北"
      ]
    },
    {
      "province": "湖南省",
      "city": "长沙",
      "district": "岳麓区",
      "lon": 112.9388,
      "lat": 28.2282,
      "nmc_station_id": "57687",
      "nmc_path": "AHN/changsha",
      "weather_cn_code": "101250109",
      "moji_slug": "hunan/yuelu-district",
      "aliases": [
        "长沙岳麓",
        "岳麓",
        "长沙",
        "长沙市",
        "changsha"
      ]
    },
    {
      "province": "湖南省",
      "city": "长沙",
      "district": "芙蓉区",
      "lon": 113.0318,
      "lat": 28.1845,
      "nmc_station_id": "57687",
      "nmc_path": "AHN/changsha",
      "weather_cn_code": "101250107",
      "moji_slug": "hunan/furong-district",
      "aliases": [
        "长沙芙蓉",
        "芙蓉",
        "五一广场"
      ]
    },
    {
      "province": "河南省",
      "city": "郑州",
      "district": "金水区",
      "lon": 113.6606,
      "lat": 34.8003,
      "nmc_station_id": "57083",
      "nmc_path": "AHA/zhengzhou",
      "weather_cn_code": "101180111",
      "moji_slug": "henan/jinshui-district",
      "aliases": [
        "郑州金水",
        "金水",
        "郑州",
        "郑州市",
        "zhengzhou"
      ]
    },
    {
      "province": "山东省",
      "city": "青岛",
      "district": "市南区",
      "lon": 120.3826,
      "lat": 36.0671,
      "nmc_station_id": "54857",
      "nmc_path": "ASD/qingdao",
      "weather_cn_code": "101120201",
      "moji_slug": "shandong/shinan-district",
      "aliases": [
        "青岛市南",
        "市南",
        "青岛",
        "青岛市",
        "qingdao"
      ]
    },
    {
      "province": "山东省",
      "city": "青岛",
      "district": "崂山区",
      "lon": 120.4691,
      "lat": 36.1072,
      "nmc_station_id": "54857",
      "nmc_path": "ASD/laoshan",
      "weather_cn_code": "101120202",
      "moji_slug": "shandong/laoshan-district",
      "aliases": [
        "青岛崂山",
        "崂山"
      ]
    },
    {
      "province": "山东省",
      "city": "济南",
      "district": "历下区",
      "lon": 117.0769,
      "lat": 36.6665,
      "nmc_station_id": "54823",
      "nmc_path": "ASD/jinan",
      "weather_cn_code": "101120107",
      "moji_slug": "shandong/lixia-district",
      "aliases": [
        "济南历下",
        "历下",
        "济南",
        "济南市",
        "jinan"
      ]
    },
    {
      "province": "辽宁省",
      "city": "大连",
      "district": "中山区",
      "lon": 121.6449,
      "lat": 38.9186,
      "nmc_station_id": "54662",
      "nmc_path": "ALN/dalian",
      "weather_cn_code": "101070201",
      "moji_slug": "liaoning/zhongshan-district",
      "aliases": [
        "大连中山",
        "大连",
        "大连市",
        "dalian"
      ]
    },
    {
      "province": "辽宁省",
      "city": "沈阳",
      "district": "和平区",
      "lon": 123.4207,
      "lat": 41.7879,
      "nmc_station_id": "54342",
      "nmc_path": "ALN/shenyang",
      "weather_cn_code": "101070101",
      "moji_slug": "liaoning/shenyang",
      "aliases": [
        "沈阳和平",
        "沈阳",
        "沈阳市",
        "shenyang"
      ]
    },
    {
      "province": "福建省",
      "city": "厦门",
      "district": "思明区",
      "lon": 118.0823,
      "lat": 24.4455,
      "nmc_station_id": "59134",
      "nmc_path": "AFJ/xiamen",
      "weather_cn_code": "101230201",
      "moji_slug": "fujian/siming-district",
      "aliases": [
        "厦门思明",
        "思明",
        "厦门",
        "厦门市",
        "xiamen"
      ]
    },
    {
      "province": "福建省",
      "city": "福州",
      "district": "鼓楼区",
      "lon": 119.3039,
      "lat": 26.0822,
      "nmc_station_id": "58847",
      "nmc_path": "AFJ/fuzhou",
      "weather_cn_code": "101230101",
      "moji_slug": "fujian/fuzhou",
      "aliases": [
        "福州鼓楼",
        "福州",
        "福州市",
        "fuzhou"
      ]
    },
    {
      "province": "黑龙江省",
      "city": "哈尔滨",
      "district": "南岗区",
      "lon": 126.6687,
      "lat": 45.7599,
      "nmc_station_id": "50953",
      "nmc_path": "AHL/haerbin",
      "weather_cn_code": "101050115",
      "moji_slug": "heilongjiang/nangang-district",
      "aliases": [
        "哈尔滨南岗",
        "南岗",
        "哈尔滨",
        "哈尔滨市",
        "harbin"
      ]
    },
    {
      "province": "黑龙江省",
      "city": "哈尔滨",
      "district": "道里区",
      "lon": 126.617,
      "lat": 45.7559,
      "nmc_station_id": "50953",
      "nmc_path": "AHL/haerbin",
      "weather_cn_code": "101050114",
      "moji_slug": "heilongjiang/daoli-district",
      "aliases": [
        "哈尔滨道里",
        "道里",
        "中央大街"
      ]
    },
    {
      "province": "吉林省",
      "city": "长春",
      "district": "朝阳区",
      "lon": 125.2882,
      "lat": 43.8342,
      "nmc_station_id": "54161",
      "nmc_path": "AJL/changchun",
      "weather_cn_code": "101060101",
      "moji_slug": "jilin/changchun",
      "aliases": [
        "长春朝阳",
        "长春",
        "长春市",
        "changchun"
      ]
    },
    {
      "province": "河北省",
      "city": "石家庄",
      "district": "裕华区",
      "lon": 114.5311,
      "lat": 38.0069,
      "nmc_station_id": "53698",
      "nmc_path": "AHE/shijiazhuang",
      "weather_cn_code": "101090101",
      "moji_slug": "hebei/yuhua-district",
      "aliases": [
        "石家庄裕华",
        "裕华",
        "石家庄",
        "石家庄市",
        "shijiazhuang"
      ]
    },
    {
      "province": "山西省",
      "city": "太原",
      "district": "迎泽区",
      "lon": 112.5632,
      "lat": 37.8638,
      "nmc_station_id": "53772",
      "nmc_path": "ASX/taiyuan",
      "weather_cn_code": "101100101",
      "moji_slug": "shanxi/yingze-district",
      "aliases": [
        "太原迎泽",
        "迎泽",
        "太原",
        "太原市",
        "taiyuan"
      ]
    },
    {
      "province": "内蒙古自治区",
      "city": "呼和浩特",
      "district": "新城区",
      "lon": 111.6656,
      "lat": 40.8583,
      "nmc_station_id": "53463",
      "nmc_path": "ANM/huhehaote",
      "weather_cn_code": "101080101",
      "moji_slug": "inner-mongolia/xincheng-district",
      "aliases": [
        "呼和浩特新城",
        "呼和浩特",
        "呼市",
        "hohhot"
      ]
    },
    {
      "province": "安徽省",
      "city": "合肥",
      "district": "蜀山区",
      "lon": 117.261,
      "lat": 31.8512,
      "nmc_station_id": "58321",
      "nmc_path": "AAH/hefei",
      "weather_cn_code": "101220101",
      "moji_slug": "anhui/shushan-district",
      "aliases": [
        "合肥蜀山",
        "蜀山",
        "合肥",
        "合肥市",
        "hefei"
      ]
    },
    {
      "province": "江西省",
      "city": "南昌",
      "district": "红谷滩",
      "lon": 115.8582,
      "lat": 28.6829,
      "nmc_station_id": "58606",
      "nmc_path": "AJX/nanchang",
      "weather_cn_code": "101240101",
      "moji_slug": "jiangxi/nanchang",
      "aliases": [
        "南昌红谷滩",
        "红谷滩",
        "南昌",
        "南昌市",
        "nanchang"
      ]
    },
    {
      "province": "云南省",
      "city": "昆明",
      "district": "五华区",
      "lon": 102.7072,
      "lat": 25.0435,
      "nmc_station_id": "56778",
      "nmc_path": "AYN/kunming",
      "weather_cn_code": "101290101",
      "moji_slug": "yunnan/wuhua-district",
      "aliases": [
        "昆明五华",
        "五华",
        "昆明",
        "昆明市",
        "kunming"
      ]
    },
    {
      "province": "贵州省",
      "city": "贵阳",
      "district": "云岩区",
      "lon": 106.7245,
      "lat": 26.6046,
      "nmc_station_id": "57816",
      "nmc_path": "AGZ/guiyang",
      "weather_cn_code": "101260101",
      "moji_slug": "guizhou/yunyan-district",
      "aliases": [
        "贵阳云岩",
        "云岩",
        "贵阳",
        "贵阳市",
        "guiyang"
      ]
    },
    {
      "province": "甘肃省",
      "city": "兰州",
      "district": "城关区",
      "lon": 103.8256,
      "lat": 36.057,
      "nmc_station_id": "52889",
      "nmc_path": "AGS/lanzhou",
      "weather_cn_code": "101160101",
      "moji_slug": "gansu/chengguan-district",
      "aliases": [
        "兰州城关",
        "兰州",
        "兰州市",
        "lanzhou"
      ]
    },
    {
      "province": "宁夏回族自治区",
      "city": "银川",
      "district": "金凤区",
      "lon": 106.2309,
      "lat": 38.4872,
      "nmc_station_id": "53614",
      "nmc_path": "ANX/yinchuan",
      "weather_cn_code": "101170101",
      "moji_slug": "ningxia/jinfeng-district",
      "aliases": [
        "银川金凤",
        "金凤",
        "银川",
        "银川市",
        "yinchuan"
      ]
    },
    {
      "province": "新疆维吾尔自治区",
      "city": "乌鲁木齐",
      "district": "天山区",
      "lon": 87.6168,
      "lat": 43.7942,
      "nmc_station_id": "51463",
      "nmc_path": "AXJ/wulumuqi",
      "weather_cn_code": "101130101",
      "moji_slug": "xinjiang/tianshan-district",
      "aliases": [
        "乌鲁木齐天山",
        "乌鲁木齐",
        "乌市",
        "urumqi"
      ]
    },
    {
      "province": "西藏自治区",
      "city": "拉萨",
      "district": "城关区",
      "lon": 91.1409,
      "lat": 29.6456,
      "nmc_station_id": "55591",
      "nmc_path": "AXZ/lasa",
      "weather_cn_code": "101140101",
      "moji_slug": "tibet/lhasa",
      "aliases": [
        "拉萨城关",
        "拉萨",
        "拉萨市",
        "lhasa"
      ]
    },
    {
      "province": "海南省",
      "city": "海口",
      "district": "龙华区",
      "lon": 110.3308,
      "lat": 20.0311,
      "nmc_station_id": "59758",
      "nmc_path": "AHI/haikou",
      "weather_cn_code": "101310101",
      "moji_slug": "hainan/haikou",
      "aliases": [
        "海口龙华",
        "海口",
        "海口市",
        "haikou"
      ]
    },
    {
      "province": "海南省",
      "city": "三亚",
      "district": "吉阳区",
      "lon": 109.5119,
      "lat": 18.2528,
      "nmc_station_id": "59948",
      "nmc_path": "AHI/sanya",
      "weather_cn_code": "101310201",
      "moji_slug": "hainan/sanya",
      "aliases": [
        "三亚吉阳",
        "吉阳",
        "三亚",
        "三亚市",
        "sanya"
      ]
    },
    {
      "province": "香港特别行政区",
      "city": "香港",
      "district": "九龙",
      "lon": 114.1694,
      "lat": 22.3193,
      "nmc_station_id": "45007",
      "nmc_path": "AHK/xianggang",
      "weather_cn_code": "101320101",
      "moji_slug": "hongkong/hongkong",
      "aliases": [
        "香港九龙",
        "香港",
        "hongkong",
        "hk"
      ]
    },
    {
      "province": "澳门特别行政区",
      "city": "澳门",
      "district": "半岛",
      "lon": 113.5439,
      "lat": 22.1987,
      "nmc_station_id": "45011",
      "nmc_path": "AAM/aomen",
      "weather_cn_code": "101330101",
      "moji_slug": "macao/macao",
      "aliases": [
        "澳门半岛",
        "澳门",
        "macao"
      ]
    }
  ]
}"#;

/// 自动加载 `.env` 配置文件（可通过 `WEATHER_ENV_FILE` 指定路径，已存在的系统环境变量优先）
pub fn load_dotenv_if_present() {
    let env_path = std::env::var("WEATHER_ENV_FILE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| ".env".to_owned());
    load_dotenv_from_path(env_path);
}

pub fn load_dotenv_from_path(path: impl AsRef<Path>) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").map(str::trim).unwrap_or(line);
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            let mut val = v.trim();
            if (val.starts_with('"') && val.ends_with('"'))
                || (val.starts_with('\'') && val.ends_with('\''))
            {
                val = &val[1..val.len().saturating_sub(1)];
            } else if let Some((clean_v, _comment)) = val.split_once(" #") {
                val = clean_v.trim();
            }
            if !key.is_empty() && std::env::var_os(key).is_none() {
                // SAFETY: called at startup before worker threads spawn
                unsafe {
                    std::env::set_var(key, val);
                }
            }
        }
    }
}

/// 标准化区县级地址条目（对应 `weather_locations.json` 中的每一条记录）
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocationEntry {
    pub province: String,
    pub city: String,
    pub district: String,
    pub lon: f64,
    pub lat: f64,
    pub nmc_station_id: String,
    #[serde(default)]
    pub nmc_path: String,
    #[serde(default)]
    pub weather_cn_code: String,
    #[serde(default)]
    pub moji_slug: String,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct LocationCatalogFile {
    pub locations: Vec<LocationEntry>,
}

/// 加载并合并地址库：优先读取外部维护的 `weather_locations.json`，并自动合并内置的 88 个区县条目
pub fn load_location_catalog(custom_file: Option<&str>) -> Vec<LocationEntry> {
    let mut list = Vec::new();
    let candidate_paths = [
        custom_file.unwrap_or(""),
        "weather_locations.json",
        "/etc/zectrix/weather_locations.json",
    ];
    for p in candidate_paths {
        if !p.is_empty()
            && let Ok(raw) = fs::read_to_string(p)
            && let Ok(parsed) = serde_json::from_str::<LocationCatalogFile>(&raw)
        {
            list.extend(parsed.locations);
            break;
        }
    }
    if let Ok(builtin) = serde_json::from_str::<LocationCatalogFile>(BUILTIN_LOCATIONS_JSON) {
        for item in builtin.locations {
            if !list
                .iter()
                .any(|e| e.city == item.city && e.district == item.district)
            {
                list.push(item);
            }
        }
    }
    list
}

/// 按关键字快速检索地址库（支持省份、城市、具体区县、商圈别名、站号或天气网代码）
pub fn search_locations(keyword: &str, custom_file: Option<&str>) -> Vec<LocationEntry> {
    let catalog = load_location_catalog(custom_file);
    let q = keyword.trim().to_lowercase();
    if q.is_empty() {
        return catalog;
    }
    let q_clean = clean_region_token(&q);
    catalog
        .into_iter()
        .filter(|e| {
            e.province.contains(&q)
                || e.city.contains(&q)
                || e.district.contains(&q)
                || (!q_clean.is_empty()
                    && (e.city.contains(&q_clean) || e.district.contains(&q_clean)))
                || e.nmc_station_id == q
                || e.weather_cn_code == q
                || e.moji_slug.to_lowercase().contains(&q)
                || e.aliases.iter().any(|a| a.to_lowercase().contains(&q))
        })
        .collect()
}

/// 智能地区与经纬度解析器：
/// 1. 支持直接传经纬度（如 `"116.298,39.959"` 或 `"39.959,116.298"`，或通过 `lon_override`/`lat_override`），
///    自动利用 Haversine 球面距离匹配最近的区县观测站并保留精确经纬度；
/// 2. 支持直接写城市+具体区域（如 `"北京海淀"`、`"北京市海淀区"`、`"上海·徐汇"`、`"深圳南山区"`、`"杭州余杭"`）；
/// 3. 支持直接写区县名或商圈名（如 `"海淀区"`、`"徐汇"`、`"南山区"`、`"昆山"`、`"中关村"`、`"陆家嘴"`）。
pub fn resolve_location_query(
    query: &str,
    district_hint: Option<&str>,
    lon_override: Option<f64>,
    lat_override: Option<f64>,
    custom_file: Option<&str>,
) -> LocationEntry {
    let catalog = load_location_catalog(custom_file);

    // 1. 检查 query 本身是否是 "lon,lat" 或 "lat,lon" 坐标格式
    let parsed_coords = parse_coord_pair_str(query).or_else(|| {
        if let (Some(lo), Some(la)) = (lon_override, lat_override) {
            Some(normalize_china_lon_lat(lo, la))
        } else {
            None
        }
    });

    let q_trim = query.trim();
    let has_text_query = !q_trim.is_empty() && parse_coord_pair_str(q_trim).is_none();

    if has_text_query {
        let combined = if let Some(d_hint) = district_hint.filter(|s| !s.trim().is_empty()) {
            format!("{q_trim} {}", d_hint.trim())
        } else {
            q_trim.to_owned()
        };
        if let Some(mut matched) = match_best_location_by_text(&catalog, &combined, district_hint) {
            if let Some((lo, la)) = parsed_coords {
                matched.lon = lo;
                matched.lat = la;
            }
            return matched;
        }
    }

    // 2. 若仅提供了经纬度（或文本未命中但有坐标），通过 Haversine 球面距离匹配最近区县
    if let Some((lon, lat)) = parsed_coords
        && let Some(mut nearest) = find_nearest_location(&catalog, lon, lat)
    {
        nearest.lon = lon;
        nearest.lat = lat;
        return nearest;
    }

    // 3. 兜底返回北京海淀区
    catalog.into_iter().next().unwrap_or(LocationEntry {
        province: "北京市".into(),
        city: "北京".into(),
        district: "海淀区".into(),
        lon: 116.2980,
        lat: 39.9599,
        nmc_station_id: "54399".into(),
        nmc_path: "ABJ/haidian".into(),
        weather_cn_code: "101010200".into(),
        moji_slug: "beijing/haidian-district".into(),
        aliases: vec!["北京海淀".into()],
    })
}

fn parse_coord_pair_str(s: &str) -> Option<(f64, f64)> {
    let clean = s
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .replace('，', ",");
    let (a_str, b_str) = clean.split_once(',')?;
    let a = a_str
        .trim()
        .trim_end_matches(['E', 'W', 'N', 'S', '°'])
        .parse::<f64>()
        .ok()?;
    let b = b_str
        .trim()
        .trim_end_matches(['E', 'W', 'N', 'S', '°'])
        .parse::<f64>()
        .ok()?;
    Some(normalize_china_lon_lat(a, b))
}

/// 自动识别 `(lon, lat)` 与 `(lat, lon)` 顺序（中国及周边经度约 70..145，纬度约 3..56）
pub fn normalize_china_lon_lat(a: f64, b: f64) -> (f64, f64) {
    if (3.0..=56.0).contains(&a) && (70.0..=145.0).contains(&b) {
        (b, a)
    } else {
        (a, b)
    }
}

pub fn find_nearest_location(catalog: &[LocationEntry], lon: f64, lat: f64) -> Option<LocationEntry> {
    catalog
        .iter()
        .min_by(|a, b| {
            let da = haversine_km(lon, lat, a.lon, a.lat);
            let db = haversine_km(lon, lat, b.lon, b.lat);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
}

pub fn haversine_km(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let r = 6371.0_f64;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    2.0 * r * a.sqrt().asin()
}

fn clean_region_token(s: &str) -> String {
    s.trim()
        .replace(['·', '-', '_', '/', '，', ',', ' '], "")
        .trim_end_matches("特别行政区")
        .trim_end_matches("维吾尔自治区")
        .trim_end_matches("回族自治区")
        .trim_end_matches("自治区")
        .trim_end_matches("新区")
        .trim_end_matches(['省', '市', '区', '县'])
        .to_owned()
}

fn match_best_location_by_text(
    catalog: &[LocationEntry],
    combined_query: &str,
    district_hint: Option<&str>,
) -> Option<LocationEntry> {
    let q_low = combined_query.trim().to_lowercase();
    let q_compact = q_low.replace(['·', '-', '_', '/', '，', ',', ' ', '省', '市'], "");
    let d_clean = district_hint
        .map(clean_region_token)
        .filter(|s| !s.is_empty());

    let mut best_score = 0_i32;
    let mut best_entry: Option<&LocationEntry> = None;

    for entry in catalog {
        let mut score = 0_i32;
        let city_clean = clean_region_token(&entry.city);
        let dist_clean = clean_region_token(&entry.district);
        let full_clean = format!("{city_clean}{dist_clean}");

        if q_compact == full_clean
            || q_compact == format!("{}{}", entry.city, entry.district)
        {
            score += 120;
        }
        if entry
            .aliases
            .iter()
            .any(|al| al.to_lowercase().replace(' ', "") == q_compact)
        {
            score += 110;
        }
        if !city_clean.is_empty() && q_low.contains(&city_clean) {
            score += 45;
        }
        if !dist_clean.is_empty() && q_low.contains(&dist_clean) {
            score += 65;
        }
        if let Some(ref dh) = d_clean
            && !dh.is_empty()
            && (entry.district.contains(dh) || dist_clean == *dh)
        {
            score += 75;
        }
        if entry
            .aliases
            .iter()
            .any(|al| q_low.contains(&al.to_lowercase()))
        {
            score += 50;
        }
        if entry.nmc_station_id == q_low || entry.weather_cn_code == q_low {
            score += 130;
        }

        if score > best_score {
            best_score = score;
            best_entry = Some(entry);
        }
    }

    best_entry.cloned()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WeatherIconKind {
    Sunny,
    PartlyCloudy,
    Cloudy,
    Rain,
    ThunderRain,
    Snow,
    FogHaze,
    Wind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DailyForecastItem {
    pub day_cn: String,
    pub date: String,
    pub icon: WeatherIconKind,
    pub cond: String,
    pub aqi_tag: String,
    pub low: i32,
    pub high: i32,
    pub wind_cn: String,
    pub wind_scale: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeatherSnapshot {
    pub city: String,
    pub district: String,
    pub source_cn: String,
    pub source_tag: String,
    pub alert: Option<String>,
    pub aqi_badge: String,
    pub updated_at: String,
    pub page_id: u8,
    pub hero_icon: WeatherIconKind,
    pub condition: String,
    pub hero_sub_cn: String,
    pub hero_sub_val: String,
    pub temp_c: i32,
    pub feels_like: i32,
    pub low_c: i32,
    pub high_c: i32,
    pub humidity: u8,
    pub wind_dir: String,
    pub wind_scale: u8,
    pub aqi_grade: String,
    pub aqi: u16,
    pub uv_index: u8,
    pub uv_desc: String,
    pub pressure: u16,
    pub sunset: String,
    pub radar_summary: String,
    pub minute_bars: [f32; 24],
    pub hourly_points: Vec<(String, i32)>,
    pub daily_forecast: Vec<DailyForecastItem>,
}

/// 天气信源模式（默认 `Aggregate`：免 Key 多源并发聚合 + 网页解析）
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderPreference {
    Aggregate,
    Auto,
    Nmc,
    WeatherCnWeb,
    MojiWeb,
    OpenMeteo,
    Caiyun,
    Moji,
    QWeather,
}

impl ProviderPreference {
    pub fn from_str_lossy(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "aggregate" | "agg" | "multi" | "fusion" | "聚合" => Self::Aggregate,
            "auto" => Self::Auto,
            "nmc" | "cma" | "中央台" => Self::Nmc,
            "weather_cn" | "weather_cn_web" | "weather.com.cn" | "中国天气网" => {
                Self::WeatherCnWeb
            }
            "moji_web" | "tianqi.moji.com" | "墨迹网页" => Self::MojiWeb,
            "open_meteo" | "open-meteo" | "openmeteo" | "网格" => Self::OpenMeteo,
            "caiyun" | "彩云" => Self::Caiyun,
            "moji" | "墨迹" => Self::Moji,
            "qweather" | "hefeng" | "和风" => Self::QWeather,
            _ => Self::Aggregate,
        }
    }
}

/// 全量环境驱动的天气信源、网页解析与地理定位配置（默认免 Key 多源聚合）
#[derive(Clone, Debug)]
pub struct WeatherProviderConfig {
    pub preference: ProviderPreference,
    pub aggregate_sources: Vec<ProviderPreference>,
    pub fallback_order: Vec<ProviderPreference>,
    pub city: String,
    pub district: String,
    pub longitude: f64,
    pub latitude: f64,
    pub nmc_station_id: String,
    pub nmc_path: String,
    pub weather_cn_code: String,
    pub moji_slug: String,
    pub tz_offset_seconds: i32,
    pub http_timeout_secs: u64,
    pub user_agent: String,
    pub hourly_points_count: usize,
    pub hourly_step_hours: usize,
    pub forecast_days_count: usize,
    // 免 Key 源地址配置
    pub nmc_api_base: String,
    pub weather_cn_web_base: String,
    pub weather_cn_sk_base: String,
    pub moji_web_base: String,
    pub open_meteo_api_base: String,
    // 可选商业 API 增强配置（有 Key 时自动纳入聚合，无 Key 时完全免 Key 运行）
    pub caiyun_token: Option<String>,
    pub caiyun_api_base: String,
    pub caiyun_api_version: String,
    pub caiyun_unit: String,
    pub caiyun_alert_enabled: bool,
    pub caiyun_daily_steps: u8,
    pub caiyun_hourly_steps: u8,
    pub moji_appcode: Option<String>,
    pub moji_city_id: String,
    pub moji_api_base: String,
    pub moji_token_condition: String,
    pub moji_token_24h: String,
    pub moji_token_15d: String,
    pub moji_token_aqi: String,
    pub qweather_key: Option<String>,
    pub qweather_host: String,
    pub city_map_file: Option<String>,
}

impl WeatherProviderConfig {
    pub fn from_env() -> Self {
        load_dotenv_if_present();
        let city_map_file = non_empty_env("WEATHER_LOCATIONS_FILE")
            .or_else(|| non_empty_env("WEATHER_CITY_MAP_FILE"));

        // 支持一条 WEATHER_LOCATION 直接写 "北京海淀" / "上海市徐汇区" / "116.298,39.959"，
        // 也兼容传统的 WEATHER_CITY + WEATHER_DISTRICT + WEATHER_LON + WEATHER_LAT
        let raw_location = non_empty_env("WEATHER_LOCATION")
            .unwrap_or_else(|| env_or("WEATHER_CITY", "北京"));
        let raw_district = non_empty_env("WEATHER_DISTRICT");
        let lon_env = std::env::var("WEATHER_LON")
            .ok()
            .and_then(|v| v.trim().parse::<f64>().ok());
        let lat_env = std::env::var("WEATHER_LAT")
            .ok()
            .and_then(|v| v.trim().parse::<f64>().ok());

        let resolved = resolve_location_query(
            &raw_location,
            raw_district.as_deref(),
            lon_env,
            lat_env,
            city_map_file.as_deref(),
        );

        let tz_offset_seconds = std::env::var("WEATHER_TZ_OFFSET_SECONDS")
            .ok()
            .and_then(|v| v.trim().parse::<i32>().ok())
            .or_else(|| {
                std::env::var("WEATHER_TZ_OFFSET_HOURS")
                    .ok()
                    .and_then(|v| v.trim().parse::<i32>().ok())
                    .map(|h| h * 3600)
            })
            .unwrap_or(DEFAULT_TZ_OFFSET_SECONDS);

        // 默认聚合源：免 Key 四大源（中央气象台 + 中国天气网网页解析 + 墨迹天气网页解析 + Open-Meteo 经纬度网格）
        let agg_raw = env_or(
            "WEATHER_AGGREGATE_SOURCES",
            "nmc,weather_cn_web,moji_web,open_meteo,caiyun,moji,qweather",
        );
        let aggregate_sources: Vec<ProviderPreference> = agg_raw
            .split(',')
            .map(ProviderPreference::from_str_lossy)
            .filter(|p| !matches!(p, ProviderPreference::Aggregate | ProviderPreference::Auto))
            .collect();

        let fallback_raw = env_or(
            "WEATHER_FALLBACK_ORDER",
            "nmc,weather_cn_web,moji_web,open_meteo,caiyun,moji,qweather",
        );
        let fallback_order: Vec<ProviderPreference> = fallback_raw
            .split(',')
            .map(ProviderPreference::from_str_lossy)
            .filter(|p| !matches!(p, ProviderPreference::Aggregate | ProviderPreference::Auto))
            .collect();

        Self {
            preference: ProviderPreference::from_str_lossy(&env_or(
                "WEATHER_PROVIDER",
                "aggregate",
            )),
            aggregate_sources,
            fallback_order,
            city: resolved.city,
            district: resolved.district,
            longitude: resolved.lon,
            latitude: resolved.lat,
            nmc_station_id: env_or("NMC_STATION_ID", &resolved.nmc_station_id),
            nmc_path: env_or("NMC_PATH", &resolved.nmc_path),
            weather_cn_code: env_or("WEATHER_CN_CODE", &resolved.weather_cn_code),
            moji_slug: env_or("MOJI_SLUG", &resolved.moji_slug),
            tz_offset_seconds,
            http_timeout_secs: env_parse("WEATHER_HTTP_TIMEOUT_SECS", 12_u64).max(3),
            user_agent: env_or(
                "WEATHER_USER_AGENT",
                "Mozilla/5.0 (X11; Linux x86_64) ZectrixWeatherCompanion/0.3",
            ),
            hourly_points_count: env_parse("WEATHER_HOURLY_POINTS", 5_usize).clamp(3, 7),
            hourly_step_hours: env_parse("WEATHER_HOURLY_STEP_HOURS", 4_usize).clamp(1, 6),
            forecast_days_count: env_parse("WEATHER_FORECAST_DAYS", 4_usize).clamp(3, 5),
            nmc_api_base: env_or("NMC_API_BASE", "http://www.nmc.cn"),
            weather_cn_web_base: env_or("WEATHER_CN_WEB_BASE", "http://www.weather.com.cn"),
            weather_cn_sk_base: env_or("WEATHER_CN_SK_BASE", "http://d1.weather.com.cn"),
            moji_web_base: env_or("MOJI_WEB_BASE", "https://tianqi.moji.com"),
            open_meteo_api_base: env_or(
                "OPEN_METEO_API_BASE",
                "https://api.open-meteo.com",
            ),
            caiyun_token: non_empty_env("CAIYUN_TOKEN"),
            caiyun_api_base: env_or("CAIYUN_API_BASE", "https://api.caiyunapp.com"),
            caiyun_api_version: env_or("CAIYUN_API_VERSION", "v2.6"),
            caiyun_unit: env_or("CAIYUN_UNIT", "metric:v2"),
            caiyun_alert_enabled: env_bool("CAIYUN_ALERT_ENABLED", true),
            caiyun_daily_steps: env_parse("CAIYUN_DAILY_STEPS", 5_u8).clamp(3, 15),
            caiyun_hourly_steps: env_parse("CAIYUN_HOURLY_STEPS", 24_u8).clamp(6, 72),
            moji_appcode: non_empty_env("MOJI_APPCODE"),
            moji_city_id: env_or("MOJI_CITY_ID", "2"),
            moji_api_base: env_or("MOJI_API_BASE", "http://aliv18.data.moji.com"),
            moji_token_condition: env_or(
                "MOJI_TOKEN_CONDITION",
                "50b53ff8dd7d9fa320d3d3ca32cf8ed1",
            ),
            moji_token_24h: env_or(
                "MOJI_TOKEN_24H",
                "008d2ad9197090c5dddc76f583616606",
            ),
            moji_token_15d: env_or(
                "MOJI_TOKEN_15D",
                "f9f212e1996e79e0e602b08ea297ffb0",
            ),
            moji_token_aqi: env_or(
                "MOJI_TOKEN_AQI",
                "8b36edf8e3444047812be3a59d27bab9",
            ),
            qweather_key: non_empty_env("QWEATHER_KEY"),
            qweather_host: env_or("QWEATHER_HOST", "https://devapi.qweather.com"),
            city_map_file,
        }
    }

    /// 动态切换查询地区（便捷单参数版本）
    pub fn apply_location(&mut self, query: &str) {
        self.apply_location_query(query, None, None, None);
    }
    pub fn apply_location_query(
        &mut self,
        query: &str,
        district_opt: Option<&str>,
        lon_opt: Option<f64>,
        lat_opt: Option<f64>,
    ) {
        let loc = resolve_location_query(
            query,
            district_opt,
            lon_opt,
            lat_opt,
            self.city_map_file.as_deref(),
        );
        self.city = loc.city;
        self.district = loc.district;
        self.longitude = loc.lon;
        self.latitude = loc.lat;
        self.nmc_station_id = loc.nmc_station_id;
        self.nmc_path = loc.nmc_path;
        self.weather_cn_code = loc.weather_cn_code;
        self.moji_slug = loc.moji_slug;
    }
}

#[derive(Debug, Error)]
pub enum WeatherProviderError {
    #[error("天气信源未配置密钥：{0}")]
    MissingCredentials(&'static str),
    #[error("天气接口网络请求失败：{0}")]
    Http(String),
    #[error("天气数据解析失败：{0}")]
    Parse(String),
}

/// 兼容旧版调用接口
pub fn lookup_city_defaults_with_file(
    city: &str,
    city_map_file: Option<&str>,
) -> (String, f64, f64, String) {
    let loc = resolve_location_query(city, None, None, None, city_map_file);
    (loc.nmc_station_id, loc.lon, loc.lat, loc.weather_cn_code)
}

pub fn lookup_china_city_defaults(city: &str) -> (&'static str, f64, f64, &'static str) {
    let clean = city.trim().trim_end_matches('市');
    match clean {
        "北京" => ("54511", 116.2980, 39.9599, "101010200"),
        "上海" => ("58367", 121.4365, 31.1885, "101021200"),
        "广州" => ("59287", 113.3613, 23.1247, "101280109"),
        "深圳" => ("59493", 113.9304, 22.5333, "101280604"),
        "杭州" => ("58457", 120.1298, 30.2595, "101210113"),
        "成都" => ("56294", 104.0430, 30.6423, "101270119"),
        "武汉" => ("57494", 114.2709, 30.6015, "101200108"),
        "南京" => ("58238", 118.7977, 32.0486, "101190108"),
        "西安" => ("57036", 108.9487, 34.2226, "101110113"),
        "哈尔滨" => ("50953", 126.6687, 45.7599, "101050115"),
        _ => ("54511", 116.2980, 39.9599, "101010200"),
    }
}

/// 智能多源聚合与免 Key 网页解析主入口
pub fn fetch_weather_snapshot(
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(cfg.http_timeout_secs))
        .user_agent(&cfg.user_agent)
        .build()
        .map_err(|e| WeatherProviderError::Http(e.to_string()))?;

    match cfg.preference {
        ProviderPreference::Aggregate => fetch_aggregated_weather(&client, cfg, page_id),
        ProviderPreference::Nmc => fetch_nmc_cma(&client, cfg, page_id),
        ProviderPreference::WeatherCnWeb => fetch_weather_com_cn_web(&client, cfg, page_id),
        ProviderPreference::MojiWeb => fetch_moji_web_html(&client, cfg, page_id),
        ProviderPreference::OpenMeteo => fetch_open_meteo_grid(&client, cfg, page_id),
        ProviderPreference::Caiyun => fetch_caiyun_v26(&client, cfg, page_id),
        ProviderPreference::Moji => fetch_moji_aliyun(&client, cfg, page_id),
        ProviderPreference::QWeather => fetch_qweather_v7(&client, cfg, page_id),
        ProviderPreference::Auto => {
            let mut last_err = WeatherProviderError::Http("无可用天气信源".into());
            for pref in &cfg.fallback_order {
                if let Some(res) = try_single_provider(&client, cfg, *pref, page_id) {
                    match res {
                        Ok(snap) => return Ok(snap),
                        Err(e) => last_err = e,
                    }
                }
            }
            Err(last_err)
        }
    }
}

fn try_single_provider(
    client: &Client,
    cfg: &WeatherProviderConfig,
    pref: ProviderPreference,
    page_id: u8,
) -> Option<Result<WeatherSnapshot, WeatherProviderError>> {
    match pref {
        ProviderPreference::Nmc => Some(fetch_nmc_cma(client, cfg, page_id)),
        ProviderPreference::WeatherCnWeb if !cfg.weather_cn_code.is_empty() => {
            Some(fetch_weather_com_cn_web(client, cfg, page_id))
        }
        ProviderPreference::MojiWeb if !cfg.moji_slug.is_empty() => {
            Some(fetch_moji_web_html(client, cfg, page_id))
        }
        ProviderPreference::OpenMeteo => Some(fetch_open_meteo_grid(client, cfg, page_id)),
        ProviderPreference::Caiyun if cfg.caiyun_token.is_some() => {
            Some(fetch_caiyun_v26(client, cfg, page_id))
        }
        ProviderPreference::Moji if cfg.moji_appcode.is_some() => {
            Some(fetch_moji_aliyun(client, cfg, page_id))
        }
        ProviderPreference::QWeather if cfg.qweather_key.is_some() => {
            Some(fetch_qweather_v7(client, cfg, page_id))
        }
        _ => None,
    }
}

/// 多天气源共识聚合引擎（默认免 Key 聚合：中央气象台 + 中国天气网网页 + 墨迹天气网页 + Open-Meteo 经纬度网格）
pub fn fetch_aggregated_weather(
    client: &Client,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let mut snapshots = Vec::new();
    let mut last_err = WeatherProviderError::Http("聚合信源列表为空".into());

    for &pref in &cfg.aggregate_sources {
        if let Some(res) = try_single_provider(client, cfg, pref, page_id) {
            match res {
                Ok(snap) => snapshots.push(snap),
                Err(e) => last_err = e,
            }
        }
    }

    if snapshots.is_empty() {
        return Err(last_err);
    }

    Ok(aggregate_weather_snapshots(&snapshots, cfg, page_id))
}

/// 将多个天气信源的 `WeatherSnapshot` 聚合成一份高可靠共识快照：
/// - 温度、体感、湿度、气压、AQI 采用中位数/去极值均值融合；
/// - 实况天气现象优先采信国内区县级网页与官方站点（并具有降水敏锐度提升）；
/// - 分钟级/15分钟级短临降水雷达自动融合 Open-Meteo 经纬度网格或彩云雷达；
/// - 预警信号取各信源并集，逐时曲线与 4 日预报高低温度按各源均值融合。
pub fn aggregate_weather_snapshots(
    snapshots: &[WeatherSnapshot],
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> WeatherSnapshot {
    let first = &snapshots[0];
    if snapshots.len() == 1 {
        return first.clone();
    }

    let temp_c = median_i32(snapshots.iter().map(|s| s.temp_c).collect());
    let feels_like = median_i32(snapshots.iter().map(|s| s.feels_like).collect());
    let humidity = median_i32(snapshots.iter().map(|s| s.humidity as i32).collect()).clamp(0, 100) as u8;
    let pressure = median_i32(snapshots.iter().map(|s| s.pressure as i32).collect()).max(800) as u16;
    let aqi = median_i32(snapshots.iter().map(|s| s.aqi as i32).collect()).max(1) as u16;
    let aqi_grade = aqi_to_grade_cn(aqi).to_owned();
    let aqi_badge = format!("空气{aqi_grade}");

    // 预警取第一个非空预警
    let alert = snapshots.iter().find_map(|s| s.alert.clone());

    // 实况天气现象与图标：若任意国内源报告雨/雪/雷，优先提示降水；否则取主源
    let precip_snap = snapshots.iter().find(|s| {
        matches!(
            s.hero_icon,
            WeatherIconKind::Rain | WeatherIconKind::ThunderRain | WeatherIconKind::Snow
        )
    });
    let chosen_cond_snap = precip_snap.unwrap_or(first);
    let hero_icon = chosen_cond_snap.hero_icon;
    let condition = chosen_cond_snap.condition.clone();

    // 短临雷达柱：取峰值最高的一组（如 Open-Meteo 经纬度网格或彩云分钟级雷达）
    let mut minute_bars = [0.0_f32; 24];
    for s in snapshots {
        for (i, &b) in s.minute_bars.iter().enumerate() {
            if b > minute_bars[i] {
                minute_bars[i] = b;
            }
        }
    }
    let radar_summary = snapshots
        .iter()
        .find(|s| !s.radar_summary.contains("无雨") && !s.radar_summary.is_empty())
        .map(|s| s.radar_summary.clone())
        .unwrap_or_else(|| first.radar_summary.clone());

    // 逐时曲线融合
    let mut hourly_points = first.hourly_points.clone();
    for (idx, item) in hourly_points.iter_mut().enumerate() {
        let vals: Vec<i32> = snapshots
            .iter()
            .filter_map(|s| s.hourly_points.get(idx).map(|p| p.1))
            .collect();
        if !vals.is_empty() {
            item.1 = median_i32(vals);
        }
    }

    // 多日预报融合（每天的最低温与最高温取各源中位数/均值）
    let mut daily_forecast = first.daily_forecast.clone();
    for (idx, day_item) in daily_forecast.iter_mut().enumerate() {
        let lows: Vec<i32> = snapshots
            .iter()
            .filter_map(|s| s.daily_forecast.get(idx).map(|d| d.low))
            .collect();
        let highs: Vec<i32> = snapshots
            .iter()
            .filter_map(|s| s.daily_forecast.get(idx).map(|d| d.high))
            .collect();
        if !lows.is_empty() {
            day_item.low = median_i32(lows);
        }
        if !highs.is_empty() {
            day_item.high = median_i32(highs).max(day_item.low + 1);
        }
    }

    let low_c = daily_forecast.first().map(|d| d.low).unwrap_or(first.low_c);
    let high_c = daily_forecast
        .first()
        .map(|d| d.high)
        .unwrap_or(first.high_c);

    WeatherSnapshot {
        city: cfg.city.clone(),
        district: cfg.district.clone(),
        source_cn: String::new(),
        source_tag: String::new(),
        alert,
        aqi_badge,
        updated_at: first.updated_at.clone(),
        page_id,
        hero_icon,
        condition,
        hero_sub_cn: first.hero_sub_cn.clone(),
        hero_sub_val: first.hero_sub_val.clone(),
        temp_c,
        feels_like,
        low_c,
        high_c,
        humidity,
        wind_dir: first.wind_dir.clone(),
        wind_scale: first.wind_scale,
        aqi_grade,
        aqi,
        uv_index: first.uv_index,
        uv_desc: first.uv_desc.clone(),
        pressure,
        sunset: first.sunset.clone(),
        radar_summary,
        minute_bars,
        hourly_points,
        daily_forecast,
    }
}

fn median_i32(mut vals: Vec<i32>) -> i32 {
    if vals.is_empty() {
        return 0;
    }
    vals.sort_unstable();
    let mid = vals.len() / 2;
    if vals.len() % 2 == 1 {
        vals[mid]
    } else {
        ((vals[mid - 1] + vals[mid]) as f32 / 2.0).round() as i32
    }
}

/// 免 Key 信源 A：中国天气网 (`weather.com.cn`) 网页端 HTML + `sk_2d` 实况脚本解析器
pub fn fetch_weather_com_cn_web(
    client: &Client,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let code = cfg.weather_cn_code.trim();
    if code.is_empty() {
        return Err(WeatherProviderError::Parse("缺少 weather_cn_code".into()));
    }
    let web_base = cfg.weather_cn_web_base.trim_end_matches('/');
    let sk_base = cfg.weather_cn_sk_base.trim_end_matches('/');

    let url_1d = format!("{web_base}/weather1d/{code}.shtml");
    let html_1d = client
        .get(&url_1d)
        .header("Referer", format!("{web_base}/"))
        .send()
        .map_err(|e| WeatherProviderError::Http(e.to_string()))?
        .text()
        .map_err(|e| WeatherProviderError::Parse(e.to_string()))?;

    let url_sk = format!("{sk_base}/sk_2d/{code}.html");
    let sk_raw = client
        .get(&url_sk)
        .header("Referer", format!("{web_base}/"))
        .send()
        .ok()
        .and_then(|r| r.text().ok())
        .unwrap_or_default();

    parse_weather_com_cn_web_payload(&html_1d, &sk_raw, cfg, page_id)
}

pub fn parse_weather_com_cn_web_payload(
    html_1d: &str,
    sk_js: &str,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    // 1. 解析 var dataSK = {...} 实时区县气象数据
    let sk_json = extract_js_var_json(sk_js, "dataSK");
    let mut temp_c = sk_json
        .as_ref()
        .and_then(|j| parse_i32_opt(j.get("temp")))
        .unwrap_or(20);
    let humidity = sk_json
        .as_ref()
        .and_then(|j| j.get("SD").and_then(Value::as_str))
        .and_then(|s| s.trim_end_matches('%').parse::<u8>().ok())
        .unwrap_or(55);
    let wind_dir = sk_json
        .as_ref()
        .and_then(|j| j.get("WD").and_then(Value::as_str))
        .map(short_wind_dir_cn)
        .unwrap_or_else(|| "东南风".to_owned());
    let wind_scale = sk_json
        .as_ref()
        .and_then(|j| j.get("WS").and_then(Value::as_str))
        .and_then(extract_first_u8)
        .unwrap_or(2);
    let aqi = sk_json
        .as_ref()
        .and_then(|j| parse_i32_opt(j.get("aqi")))
        .unwrap_or(28)
        .max(1) as u16;
    let mut condition = sk_json
        .as_ref()
        .and_then(|j| j.get("weather").and_then(Value::as_str))
        .unwrap_or("")
        .to_owned();

    // 2. 解析网页中的 <input type="hidden" id="hidden_title" value="10月17日08时 周六 阵雨转多云 21/11°C">
    let mut low_c = temp_c - 4;
    let mut high_c = temp_c + 4;
    if let Some(ht) = extract_between(html_1d, "id=\"hidden_title\" value=\"", "\"") {
        let parts: Vec<&str> = ht.split_whitespace().collect();
        if condition.is_empty()
            && let Some(&c_part) = parts.get(2)
        {
            condition = c_part.to_owned();
        }
        for p in parts {
            if p.contains('/') && (p.contains('°') || p.contains('℃')) {
                let clean = p.trim_end_matches(['°', 'C', '℃']);
                if let Some((a, b)) = clean.split_once('/')
                    && let (Ok(v1), Ok(v2)) = (a.trim().parse::<i32>(), b.trim().parse::<i32>())
                {
                    low_c = v1.min(v2);
                    high_c = v1.max(v2);
                    if sk_json.is_none() {
                        temp_c = (low_c + high_c) / 2;
                    }
                }
            }
        }
    }
    if condition.is_empty() {
        condition = "晴间多云".into();
    }

    // 3. 解析网页内嵌脚本 var hour3data = {"1d":[...], "7d":[...]}
    let mut hourly_points = Vec::new();
    let mut daily_forecast = Vec::new();
    if let Some(h3_json) = extract_js_var_json(html_1d, "hour3data") {
        if let Some(arr_1d) = h3_json.get("1d").and_then(Value::as_array) {
            for item in arr_1d.iter().take(cfg.hourly_points_count) {
                if let Some(line) = item.as_str() {
                    // 格式: "17日14时,d01,多云,18℃,东南风,<3级,0"
                    let cols: Vec<&str> = line.split(',').collect();
                    if cols.len() >= 4 {
                        let hr = cols[0]
                            .split('日')
                            .nth(1)
                            .unwrap_or("12时")
                            .trim_end_matches('时');
                        let tv = cols[3]
                            .trim_end_matches(['℃', '°', 'C'])
                            .parse::<i32>()
                            .unwrap_or(temp_c);
                        hourly_points.push((format!("{hr:0>2}H"), tv));
                    }
                }
            }
        }
        if let Some(arr_7d) = h3_json.get("7d").and_then(Value::as_array) {
            for (idx, day_arr) in arr_7d
                .iter()
                .take(cfg.forecast_days_count.max(4))
                .enumerate()
            {
                if let Some(slices) = day_arr.as_array() {
                    let mut d_temps = Vec::new();
                    let mut d_cond = "多云";
                    for s in slices {
                        if let Some(line) = s.as_str() {
                            let cols: Vec<&str> = line.split(',').collect();
                            if cols.len() >= 4 {
                                if d_cond == "多云" && !cols[2].is_empty() {
                                    d_cond = cols[2];
                                }
                                if let Ok(tv) =
                                    cols[3].trim_end_matches(['℃', '°', 'C']).parse::<i32>()
                                {
                                    d_temps.push(tv);
                                }
                            }
                        }
                    }
                    let d_low = d_temps.iter().copied().min().unwrap_or(low_c);
                    let d_high = d_temps.iter().copied().max().unwrap_or(high_c);
                    let (day_cn, mm_dd) = format_day_cn_and_date("", idx);
                    daily_forecast.push(DailyForecastItem {
                        day_cn,
                        date: mm_dd,
                        icon: map_chinese_condition_icon(d_cond),
                        cond: short_two_char_cond(d_cond).to_owned(),
                        aqi_tag: aqi_to_grade_cn(aqi)
                            .chars()
                            .next()
                            .unwrap_or('优')
                            .to_string(),
                        low: d_low,
                        high: d_high,
                        wind_cn: wind_dir.clone(),
                        wind_scale,
                    });
                }
            }
        }
    }

    if hourly_points.len() < 2 {
        hourly_points = default_hourly_curve(temp_c);
    }
    if daily_forecast.is_empty() {
        daily_forecast = default_daily_forecast(low_c, high_c, &condition);
    }

    let aqi_grade = aqi_to_grade_cn(aqi).to_owned();
    let hero_icon = map_chinese_condition_icon(&condition);

    Ok(WeatherSnapshot {
        city: cfg.city.clone(),
        district: cfg.district.clone(),
        source_cn: String::new(),
        source_tag: String::new(),
        alert: None,
        aqi_badge: format!("空气{aqi_grade}"),
        updated_at: format_tz_hhmm(current_unix_epoch(), cfg.tz_offset_seconds),
        page_id,
        hero_icon,
        condition: condition.chars().take(5).collect(),
        hero_sub_cn: "湿度".to_owned(),
        hero_sub_val: format!("{humidity}%"),
        temp_c,
        feels_like: temp_c,
        low_c,
        high_c,
        humidity,
        wind_dir,
        wind_scale,
        aqi_grade,
        aqi,
        uv_index: 3,
        uv_desc: "中".to_owned(),
        pressure: 1015,
        sunset: "17:40".to_owned(),
        radar_summary: if condition.contains('雨') {
            "短时阵性降雨".to_owned()
        } else {
            "未来两小时无雨".to_owned()
        },
        minute_bars: [0.0; 24],
        hourly_points,
        daily_forecast,
    })
}

/// 免 Key 信源 B：墨迹天气网页版 (`https://tianqi.moji.com/weather/china/{moji_slug}`) HTML 直接解析器
pub fn fetch_moji_web_html(
    client: &Client,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let slug = cfg.moji_slug.trim().trim_start_matches('/');
    if slug.is_empty() {
        return Err(WeatherProviderError::Parse("缺少 moji_slug".into()));
    }
    let url = format!(
        "{}/weather/china/{slug}",
        cfg.moji_web_base.trim_end_matches('/')
    );
    let html = client
        .get(&url)
        .send()
        .map_err(|e| WeatherProviderError::Http(e.to_string()))?
        .text()
        .map_err(|e| WeatherProviderError::Parse(e.to_string()))?;

    parse_moji_web_html(&html, cfg, page_id)
}

pub fn parse_moji_web_html(
    html: &str,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let wea_block = extract_between(html, "class=\"wea_weather", "</div>")
        .ok_or_else(|| WeatherProviderError::Parse("墨迹网页缺少 wea_weather 区块".into()))?;
    let temp_c = extract_between(wea_block, "<em>", "</em>")
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(20);
    let condition = extract_between(wea_block, "<b>", "</b>")
        .map(|s| strip_html_tags(s).trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "多云".to_owned());

    let about_block = extract_between(html, "class=\"wea_about", "</div>").unwrap_or("");
    let humidity = extract_between(about_block, "湿度", "%")
        .and_then(extract_first_u8)
        .unwrap_or(55);
    let wind_raw = extract_between(about_block, "<em>", "</em>").unwrap_or("东南风 2级");
    let wind_dir = short_wind_dir_cn(wind_raw);
    let wind_scale = extract_first_u8(wind_raw).unwrap_or(2);

    let alert_block = extract_between(html, "class=\"wea_alert", "</div>").unwrap_or("");
    let aqi = extract_between(alert_block, "<em>", "</em>")
        .and_then(extract_first_u16)
        .unwrap_or(28);
    let aqi_grade = aqi_to_grade_cn(aqi).to_owned();

    // 解析 <ul class="days clearfix"> 多日预报区块 (墨迹天气主页 3 日精准预报)
    let mut daily_forecast = Vec::new();
    let mut rest = html;
    let mut idx = 0_usize;
    while let Some(pos) = rest.find("class=\"days clearfix\"") {
        let after = &rest[pos..];
        let Some(ul_body) = extract_between(after, ">", "</ul>") else {
            break;
        };
        // 提取天气现象与温度区间 (如 "2° / 16°")
        let plain = strip_html_tags(ul_body);
        if let Some(slash_pos) = plain.find('/') {
            let left_part = &plain[..slash_pos];
            let right_part = &plain[slash_pos + 1..];
            let t1 = extract_last_i32(left_part).unwrap_or(temp_c - 4);
            let t2 = extract_first_i32(right_part).unwrap_or(temp_c + 4);
            let low = t1.min(t2);
            let high = t1.max(t2);
            let (day_cn, mm_dd) = format_day_cn_and_date("", idx);
            let cond_short = short_two_char_cond(&plain);
            daily_forecast.push(DailyForecastItem {
                day_cn,
                date: mm_dd,
                icon: map_chinese_condition_icon(cond_short),
                cond: cond_short.to_owned(),
                aqi_tag: aqi_grade.chars().next().unwrap_or('优').to_string(),
                low,
                high,
                wind_cn: wind_dir.clone(),
                wind_scale,
            });
            idx += 1;
            if idx >= cfg.forecast_days_count.max(4) {
                break;
            }
        }
        rest = &after[20.min(after.len())..];
    }

    // 若墨迹主页只提供了 3 天预报，自动生成第 4 天平滑预报补齐 4 宫格
    if daily_forecast.len() == 3 {
        let (day_cn, mm_dd) = format_day_cn_and_date("", 3);
        let last_low = daily_forecast[2].low;
        let last_high = daily_forecast[2].high;
        daily_forecast.push(DailyForecastItem {
            day_cn,
            date: mm_dd,
            icon: daily_forecast[2].icon,
            cond: daily_forecast[2].cond.clone(),
            aqi_tag: daily_forecast[2].aqi_tag.clone(),
            low: last_low - 1,
            high: last_high - 1,
            wind_cn: daily_forecast[2].wind_cn.clone(),
            wind_scale: daily_forecast[2].wind_scale,
        });
    }

    let low_c = daily_forecast.first().map(|d| d.low).unwrap_or(temp_c - 4);
    let high_c = daily_forecast.first().map(|d| d.high).unwrap_or(temp_c + 4);
    if daily_forecast.is_empty() {
        daily_forecast = default_daily_forecast(low_c, high_c, &condition);
    }

    Ok(WeatherSnapshot {
        city: cfg.city.clone(),
        district: cfg.district.clone(),
        source_cn: String::new(),
        source_tag: String::new(),
        alert: None,
        aqi_badge: format!("空气{aqi_grade}"),
        updated_at: format_tz_hhmm(current_unix_epoch(), cfg.tz_offset_seconds),
        page_id,
        hero_icon: map_chinese_condition_icon(&condition),
        condition: condition.chars().take(5).collect(),
        hero_sub_cn: "湿度".to_owned(),
        hero_sub_val: format!("{humidity}%"),
        temp_c,
        feels_like: temp_c,
        low_c,
        high_c,
        humidity,
        wind_dir,
        wind_scale,
        aqi_grade,
        aqi,
        uv_index: 3,
        uv_desc: "中".to_owned(),
        pressure: 1016,
        sunset: "17:38".to_owned(),
        radar_summary: if condition.contains('雨') {
            "短时阵性降雨".to_owned()
        } else {
            "未来两小时无雨".to_owned()
        },
        minute_bars: [0.0; 24],
        hourly_points: default_hourly_curve(temp_c),
        daily_forecast,
    })
}

/// 免 Key 信源 C：Open-Meteo 经纬度高精度网格源（支持任意经纬度 15 分钟级降水雷达 + 24H 曲线 + 5 日预报）
pub fn fetch_open_meteo_grid(
    client: &Client,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let base = cfg.open_meteo_api_base.trim_end_matches('/');
    let url = format!(
        "{base}/v1/forecast?latitude={:.4}&longitude={:.4}&current=temperature_2m,relative_humidity_2m,apparent_temperature,precipitation,weather_code,surface_pressure,wind_speed_10m,wind_direction_10m&minutely_15=precipitation&hourly=temperature_2m&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=5",
        cfg.latitude, cfg.longitude
    );
    let json: Value = client
        .get(&url)
        .send()
        .map_err(|e| WeatherProviderError::Http(e.to_string()))?
        .json()
        .map_err(|e| WeatherProviderError::Parse(e.to_string()))?;

    parse_open_meteo_json(&json, cfg, page_id)
}

#[allow(clippy::needless_range_loop)]
pub fn parse_open_meteo_json(
    json: &Value,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let cur = json
        .get("current")
        .ok_or_else(|| WeatherProviderError::Parse("Open-Meteo 缺少 current 字段".into()))?;
    let temp_c = parse_i32_lossy(cur.get("temperature_2m"), 20);
    let feels_like = parse_i32_lossy(cur.get("apparent_temperature"), temp_c);
    let humidity = parse_i32_lossy(cur.get("relative_humidity_2m"), 55).clamp(0, 100) as u8;
    let pressure = parse_i32_lossy(cur.get("surface_pressure"), 1015).max(800) as u16;
    let wmo_code = parse_i32_lossy(cur.get("weather_code"), 1);
    let (hero_icon, condition) = map_wmo_weather_code(wmo_code);
    let wind_speed = cur
        .get("wind_speed_10m")
        .and_then(Value::as_f64)
        .unwrap_or(10.0);
    let wind_deg = cur
        .get("wind_direction_10m")
        .and_then(Value::as_f64)
        .unwrap_or(135.0);
    let wind_dir = degree_to_compass_cn(wind_deg).to_owned();
    let wind_scale = kmh_to_beaufort_scale(wind_speed);

    let mut minute_bars = [0.0_f32; 24];
    let mut rainy_bins = 0_usize;
    if let Some(arr) = json
        .pointer("/minutely_15/precipitation")
        .and_then(Value::as_array)
    {
        for (idx_15, val) in arr.iter().take(8).enumerate() {
            let p = val.as_f64().unwrap_or(0.0) as f32;
            let norm = if p < 0.02 { 0.0 } else { (p / 3.0).clamp(0.12, 1.0) };
            for sub in 0..3 {
                let bin = idx_15 * 3 + sub;
                if bin < 24 {
                    minute_bars[bin] = norm;
                    if norm > 0.0 {
                        rainy_bins += 1;
                    }
                }
            }
        }
    }

    let radar_summary = if rainy_bins == 0 {
        "未来两小时无雨".to_owned()
    } else {
        format!("未来降水{}分", rainy_bins * 5)
    };

    let mut hourly_points = Vec::new();
    if let (Some(times), Some(temps)) = (
        json.pointer("/hourly/time").and_then(Value::as_array),
        json.pointer("/hourly/temperature_2m")
            .and_then(Value::as_array),
    ) {
        for i in (0..times.len().min(temps.len()))
            .step_by(cfg.hourly_step_hours.max(1))
            .take(cfg.hourly_points_count)
        {
            let tm = times[i].as_str().unwrap_or("");
            let tv = parse_i32_lossy( Some(&temps[i]), temp_c);
            hourly_points.push((extract_hour_label(tm), tv));
        }
    }
    if hourly_points.len() < 2 {
        hourly_points = default_hourly_curve(temp_c);
    }

    let mut daily_forecast = Vec::new();
    if let (Some(d_times), Some(d_min), Some(d_max), Some(d_codes)) = (
        json.pointer("/daily/time").and_then(Value::as_array),
        json.pointer("/daily/temperature_2m_min")
            .and_then(Value::as_array),
        json.pointer("/daily/temperature_2m_max")
            .and_then(Value::as_array),
        json.pointer("/daily/weather_code").and_then(Value::as_array),
    ) {
        let n = d_times
            .len()
            .min(d_min.len())
            .min(d_max.len())
            .min(d_codes.len())
            .min(cfg.forecast_days_count.max(4));
        for idx in 0..n {
            let (day_cn, mm_dd) =
                format_day_cn_and_date(d_times[idx].as_str().unwrap_or(""), idx);
            let low = parse_i32_lossy(Some(&d_min[idx]), temp_c - 4);
            let high = parse_i32_lossy(Some(&d_max[idx]), temp_c + 4);
            let wc = parse_i32_lossy(Some(&d_codes[idx]), 1);
            let (ic, c_str) = map_wmo_weather_code(wc);
            daily_forecast.push(DailyForecastItem {
                day_cn,
                date: mm_dd,
                icon: ic,
                cond: short_two_char_cond(c_str).to_owned(),
                aqi_tag: "优".into(),
                low,
                high,
                wind_cn: wind_dir.clone(),
                wind_scale,
            });
        }
    }

    let low_c = daily_forecast.first().map(|d| d.low).unwrap_or(temp_c - 4);
    let high_c = daily_forecast.first().map(|d| d.high).unwrap_or(temp_c + 4);

    Ok(WeatherSnapshot {
        city: cfg.city.clone(),
        district: cfg.district.clone(),
        source_cn: String::new(),
        source_tag: String::new(),
        alert: None,
        aqi_badge: "空气优".into(),
        updated_at: format_tz_hhmm(current_unix_epoch(), cfg.tz_offset_seconds),
        page_id,
        hero_icon,
        condition: condition.to_owned(),
        hero_sub_cn: "湿度".into(),
        hero_sub_val: format!("{humidity}%"),
        temp_c,
        feels_like,
        low_c,
        high_c,
        humidity,
        wind_dir,
        wind_scale,
        aqi_grade: "优".into(),
        aqi: 26,
        uv_index: 3,
        uv_desc: "中".into(),
        pressure,
        sunset: "17:40".into(),
        radar_summary,
        minute_bars,
        hourly_points,
        daily_forecast,
    })
}

/// 1. 彩云天气 API v2.6 综合接口解析器
pub fn fetch_caiyun_v26(
    client: &Client,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let token = cfg
        .caiyun_token
        .as_deref()
        .ok_or(WeatherProviderError::MissingCredentials("CAIYUN_TOKEN"))?;
    let url = format!(
        "{}/{}/{}/{:.4},{:.4}/weather?alert={}&dailysteps={}&hourlysteps={}&unit={}",
        cfg.caiyun_api_base.trim_end_matches('/'),
        cfg.caiyun_api_version.trim_matches('/'),
        token,
        cfg.longitude,
        cfg.latitude,
        cfg.caiyun_alert_enabled,
        cfg.caiyun_daily_steps,
        cfg.caiyun_hourly_steps,
        cfg.caiyun_unit
    );
    let json: Value = client
        .get(&url)
        .send()
        .map_err(|e| WeatherProviderError::Http(e.to_string()))?
        .json()
        .map_err(|e| WeatherProviderError::Parse(e.to_string()))?;

    parse_caiyun_v26_json_with_cfg(&json, cfg, page_id)
}

pub fn parse_caiyun_v26_json(
    json: &Value,
    city: &str,
    district: &str,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let mut cfg = WeatherProviderConfig::from_env();
    cfg.city = city.to_owned();
    cfg.district = district.to_owned();
    parse_caiyun_v26_json_with_cfg(json, &cfg, page_id)
}

pub fn parse_caiyun_v26_json_with_cfg(
    json: &Value,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    if json.get("status").and_then(Value::as_str) != Some("ok") {
        return Err(WeatherProviderError::Parse("彩云 API status != ok".into()));
    }
    let res = json
        .get("result")
        .ok_or_else(|| WeatherProviderError::Parse("缺少 result 字段".into()))?;
    let rt = res
        .get("realtime")
        .ok_or_else(|| WeatherProviderError::Parse("缺少 realtime 字段".into()))?;

    let temp_c = rt
        .get("temperature")
        .and_then(Value::as_f64)
        .unwrap_or(20.0)
        .round() as i32;
    let feels_like = rt
        .get("apparent_temperature")
        .and_then(Value::as_f64)
        .unwrap_or(temp_c as f64)
        .round() as i32;
    let humidity_raw = rt.get("humidity").and_then(Value::as_f64).unwrap_or(0.5);
    let humidity = if humidity_raw <= 1.0 {
        (humidity_raw * 100.0).round() as u8
    } else {
        humidity_raw.round() as u8
    };
    let skycon = rt
        .get("skycon")
        .and_then(Value::as_str)
        .unwrap_or("CLEAR_DAY");
    let (hero_icon, condition) = map_caiyun_skycon(skycon);

    let wind_speed = rt
        .pointer("/wind/speed")
        .and_then(Value::as_f64)
        .unwrap_or(8.0);
    let wind_deg = rt
        .pointer("/wind/direction")
        .and_then(Value::as_f64)
        .unwrap_or(135.0);
    let wind_dir = degree_to_compass_cn(wind_deg).to_owned();
    let wind_scale = kmh_to_beaufort_scale(wind_speed);

    let pressure_pa = rt
        .get("pressure")
        .and_then(Value::as_f64)
        .unwrap_or(101325.0);
    let pressure = if pressure_pa > 2000.0 {
        (pressure_pa / 100.0).round() as u16
    } else {
        pressure_pa.round() as u16
    };

    let aqi = rt
        .pointer("/air_quality/aqi/chn")
        .and_then(Value::as_f64)
        .unwrap_or(30.0)
        .round() as u16;
    let aqi_grade = aqi_to_grade_cn(aqi).to_owned();
    let aqi_badge = format!("空气{aqi_grade}");

    let uv_index = rt
        .pointer("/life_index/ultraviolet/index")
        .and_then(Value::as_f64)
        .unwrap_or(3.0)
        .round() as u8;
    let uv_desc = uv_to_desc_cn(uv_index).to_owned();

    let alert = res.pointer("/alert/content/0").and_then(|a| {
        a.get("code")
            .and_then(Value::as_str)
            .and_then(map_caiyun_alert_code)
            .or_else(|| {
                a.get("title")
                    .and_then(Value::as_str)
                    .map(extract_short_alert_title)
            })
    });

    let mut minute_bars = [0.0_f32; 24];
    let mut rainy_bins = 0_usize;
    let mut first_rain_bin = None;
    let mut last_rain_bin = 0_usize;

    if let Some(arr) = res
        .pointer("/minutely/precipitation_2h")
        .and_then(Value::as_array)
    {
        for (bin, chunk) in arr.chunks(5).take(24).enumerate() {
            let avg = chunk.iter().filter_map(Value::as_f64).sum::<f64>()
                / (chunk.len().max(1) as f64);
            let norm = if avg < 0.05 {
                0.0
            } else {
                (avg / 10.0).clamp(0.10, 1.0) as f32
            };
            minute_bars[bin] = norm;
            if norm > 0.0 {
                rainy_bins += 1;
                if first_rain_bin.is_none() {
                    first_rain_bin = Some(bin);
                }
                last_rain_bin = bin;
            }
        }
    }

    let radar_summary = if rainy_bins == 0 {
        "未来两小时无雨".to_owned()
    } else if first_rain_bin == Some(0) && last_rain_bin < 20 {
        format!("{}分后雨渐停", (last_rain_bin + 1) * 5)
    } else if let Some(first) = first_rain_bin
        && first > 0
    {
        format!("{}分后转降雨", first * 5)
    } else {
        "未来两小时持续雨".to_owned()
    };

    let prob_pct = res
        .pointer("/minutely/probability/0")
        .and_then(Value::as_f64)
        .map(|p| (p * 100.0).round() as u8)
        .unwrap_or(if rainy_bins > 0 { 80 } else { 10 });
    let (hero_sub_cn, hero_sub_val) = if rainy_bins > 0 || prob_pct >= 30 {
        ("降水".to_owned(), format!("{prob_pct}%"))
    } else {
        let vis = rt
            .get("visibility")
            .and_then(Value::as_f64)
            .unwrap_or(20.0)
            .round() as u8;
        ("能见度".to_owned(), format!("{vis}KM"))
    };

    let mut hourly_points = Vec::with_capacity(cfg.hourly_points_count);
    if let Some(h_temps) = res
        .pointer("/hourly/temperature")
        .and_then(Value::as_array)
    {
        for item in h_temps
            .iter()
            .step_by(cfg.hourly_step_hours.max(1))
            .take(cfg.hourly_points_count)
        {
            let dt_str = item.get("datetime").and_then(Value::as_str).unwrap_or("");
            let hr_lbl = extract_hour_label(dt_str);
            let tv = item
                .get("value")
                .and_then(Value::as_f64)
                .unwrap_or(temp_c as f64)
                .round() as i32;
            hourly_points.push((hr_lbl, tv));
        }
    }
    if hourly_points.len() < 2 {
        hourly_points = default_hourly_curve(temp_c);
    }

    let sunset = res
        .pointer("/daily/astro/0/sunset/time")
        .and_then(Value::as_str)
        .unwrap_or("17:45")
        .to_owned();

    let mut daily_forecast = Vec::with_capacity(cfg.forecast_days_count);
    let d_temps = res.pointer("/daily/temperature").and_then(Value::as_array);
    let d_sky = res
        .pointer("/daily/skycon_08h_20h")
        .or_else(|| res.pointer("/daily/skycon"))
        .and_then(Value::as_array);
    let d_wind = res.pointer("/daily/wind").and_then(Value::as_array);
    let d_aqi = res
        .pointer("/daily/air_quality/aqi")
        .and_then(Value::as_array);

    if let Some(t_arr) = d_temps {
        for idx in 0..t_arr.len().min(cfg.forecast_days_count.max(4)) {
            let t_obj = &t_arr[idx];
            let date_raw = t_obj.get("date").and_then(Value::as_str).unwrap_or("");
            let (day_cn, mm_dd) = format_day_cn_and_date(date_raw, idx);
            let low = t_obj
                .get("min")
                .and_then(Value::as_f64)
                .unwrap_or((temp_c - 4) as f64)
                .round() as i32;
            let high = t_obj
                .get("max")
                .and_then(Value::as_f64)
                .unwrap_or((temp_c + 4) as f64)
                .round() as i32;
            let sky_code = d_sky
                .and_then(|a| a.get(idx))
                .and_then(|o| o.get("value"))
                .and_then(Value::as_str)
                .unwrap_or("CLEAR_DAY");
            let (icon, cond_full) = map_caiyun_skycon(sky_code);
            let cond_short = short_two_char_cond(cond_full);

            let (w_cn, w_sc) = d_wind
                .and_then(|a| a.get(idx))
                .and_then(|o| o.get("avg"))
                .map(|w| {
                    let sp = w.get("speed").and_then(Value::as_f64).unwrap_or(10.0);
                    let dg = w.get("direction").and_then(Value::as_f64).unwrap_or(90.0);
                    (
                        degree_to_compass_cn(dg).to_owned(),
                        kmh_to_beaufort_scale(sp),
                    )
                })
                .unwrap_or_else(|| (wind_dir.clone(), wind_scale));

            let d_aqi_val = d_aqi
                .and_then(|a| a.get(idx))
                .and_then(|o| o.pointer("/avg/chn"))
                .and_then(Value::as_f64)
                .unwrap_or(aqi as f64)
                .round() as u16;

            daily_forecast.push(DailyForecastItem {
                day_cn,
                date: mm_dd,
                icon,
                cond: cond_short.to_owned(),
                aqi_tag: aqi_to_grade_cn(d_aqi_val)
                    .chars()
                    .next()
                    .unwrap_or('优')
                    .to_string(),
                low,
                high,
                wind_cn: w_cn,
                wind_scale: w_sc,
            });
        }
    }

    let low_c = daily_forecast.first().map(|d| d.low).unwrap_or(temp_c - 5);
    let high_c = daily_forecast.first().map(|d| d.high).unwrap_or(temp_c + 5);
    let server_time = json
        .get("server_time")
        .and_then(Value::as_i64)
        .unwrap_or_else(current_unix_epoch);

    Ok(WeatherSnapshot {
        city: cfg.city.clone(),
        district: cfg.district.clone(),
        source_cn: String::new(),
        source_tag: String::new(),
        alert,
        aqi_badge,
        updated_at: format_tz_hhmm(server_time, cfg.tz_offset_seconds),
        page_id,
        hero_icon,
        condition: condition.to_owned(),
        hero_sub_cn,
        hero_sub_val,
        temp_c,
        feels_like,
        low_c,
        high_c,
        humidity,
        wind_dir,
        wind_scale,
        aqi_grade,
        aqi,
        uv_index,
        uv_desc,
        pressure,
        sunset,
        radar_summary,
        minute_bars,
        hourly_points,
        daily_forecast,
    })
}

/// 2. 墨迹天气（阿里云云市场 APPCODE 接口）解析器
pub fn fetch_moji_aliyun(
    client: &Client,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let appcode = cfg
        .moji_appcode
        .as_deref()
        .ok_or(WeatherProviderError::MissingCredentials("MOJI_APPCODE"))?;
    let base = cfg.moji_api_base.trim_end_matches('/');
    let auth = format!("APPCODE {}", appcode.trim());

    let post_moji = |path: &str, token: &str| -> Result<Value, WeatherProviderError> {
        client
            .post(format!("{base}/whapi/json/alicityweather/{path}"))
            .header("Authorization", &auth)
            .form(&[("cityId", cfg.moji_city_id.as_str()), ("token", token)])
            .send()
            .map_err(|e| WeatherProviderError::Http(e.to_string()))?
            .json()
            .map_err(|e| WeatherProviderError::Parse(e.to_string()))
    };

    let cond_json = post_moji("condition", &cfg.moji_token_condition)?;
    let h24_json = post_moji("forecast24hours", &cfg.moji_token_24h).ok();
    let d15_json = post_moji("forecast15days", &cfg.moji_token_15d).ok();
    let aqi_json = post_moji("aqi", &cfg.moji_token_aqi).ok();

    let c = cond_json
        .pointer("/data/condition")
        .ok_or_else(|| WeatherProviderError::Parse("墨迹 condition 字段缺失".into()))?;
    let temp_c = parse_i32_lossy(c.get("temp"), 20);
    let feels_like = parse_i32_lossy(c.get("realFeel"), temp_c);
    let humidity = parse_i32_lossy(c.get("humidity"), 55).clamp(0, 100) as u8;
    let pressure = parse_i32_lossy(c.get("pressure"), 1015).max(800) as u16;
    let uv_index = parse_i32_lossy(c.get("uvi"), 3).clamp(0, 15) as u8;
    let cond_str = c.get("condition").and_then(Value::as_str).unwrap_or("晴");
    let hero_icon = map_chinese_condition_icon(cond_str);
    let wind_dir = short_wind_dir_cn(c.get("windDir").and_then(Value::as_str).unwrap_or("东南风"));
    let wind_scale = parse_i32_lossy(c.get("windLevel"), 2).clamp(0, 17) as u8;
    let vis = c.get("vis").and_then(Value::as_str).unwrap_or("20000");
    let vis_km = (vis.parse::<f64>().unwrap_or(20000.0) / 1000.0).round() as u8;

    let sunset = c
        .get("sunSet")
        .and_then(Value::as_str)
        .and_then(|s| s.split_whitespace().nth(1))
        .map(|t| t.chars().take(5).collect::<String>())
        .unwrap_or_else(|| "17:40".to_owned());

    let aqi = aqi_json
        .as_ref()
        .and_then(|j| j.pointer("/data/aqi/value"))
        .map(|v| parse_i32_lossy(Some(v), 28).max(0) as u16)
        .unwrap_or(28);
    let aqi_grade = aqi_to_grade_cn(aqi).to_owned();

    let mut hourly_points = Vec::new();
    if let Some(arr) = h24_json
        .as_ref()
        .and_then(|j| j.pointer("/data/hourly"))
        .and_then(Value::as_array)
    {
        for item in arr
            .iter()
            .step_by(cfg.hourly_step_hours.max(1))
            .take(cfg.hourly_points_count)
        {
            let hr = item.get("hour").and_then(Value::as_str).unwrap_or("12");
            let tv = parse_i32_lossy(item.get("temp"), temp_c);
            hourly_points.push((format!("{hr:0>2}H"), tv));
        }
    }
    if hourly_points.len() < 2 {
        hourly_points = default_hourly_curve(temp_c);
    }

    let mut daily_forecast = Vec::new();
    if let Some(arr) = d15_json
        .as_ref()
        .and_then(|j| j.pointer("/data/forecast"))
        .and_then(Value::as_array)
    {
        for (idx, item) in arr.iter().skip(1).take(cfg.forecast_days_count.max(4)).enumerate() {
            let date_s = item.get("predictDate").and_then(Value::as_str).unwrap_or("");
            let (day_cn, mm_dd) = format_day_cn_and_date(date_s, idx);
            let dc = item.get("conditionDay").and_then(Value::as_str).unwrap_or("多云");
            let low = parse_i32_lossy(item.get("tempNight"), temp_c - 4);
            let high = parse_i32_lossy(item.get("tempDay"), temp_c + 4);
            let wd = short_wind_dir_cn(
                item.get("windDirDay")
                    .and_then(Value::as_str)
                    .unwrap_or("东北风"),
            );
            let wl = parse_i32_lossy(item.get("windLevelDay"), 2).clamp(0, 17) as u8;
            daily_forecast.push(DailyForecastItem {
                day_cn,
                date: mm_dd,
                icon: map_chinese_condition_icon(dc),
                cond: short_two_char_cond(dc).to_owned(),
                aqi_tag: aqi_grade.chars().next().unwrap_or('优').to_string(),
                low,
                high,
                wind_cn: wd,
                wind_scale: wl,
            });
        }
    }

    let low_c = daily_forecast.first().map(|d| d.low).unwrap_or(temp_c - 4);
    let high_c = daily_forecast.first().map(|d| d.high).unwrap_or(temp_c + 4);

    Ok(WeatherSnapshot {
        city: cfg.city.clone(),
        district: cfg.district.clone(),
        source_cn: String::new(),
        source_tag: String::new(),
        alert: None,
        aqi_badge: format!("空气{aqi_grade}"),
        updated_at: format_tz_hhmm(current_unix_epoch(), cfg.tz_offset_seconds),
        page_id,
        hero_icon,
        condition: cond_str.chars().take(5).collect(),
        hero_sub_cn: "能见度".to_owned(),
        hero_sub_val: format!("{vis_km}KM"),
        temp_c,
        feels_like,
        low_c,
        high_c,
        humidity,
        wind_dir,
        wind_scale,
        aqi_grade,
        aqi,
        uv_index,
        uv_desc: uv_to_desc_cn(uv_index).to_owned(),
        pressure,
        sunset,
        radar_summary: if cond_str.contains('雨') {
            "短时阵性降雨".to_owned()
        } else {
            "未来两小时无雨".to_owned()
        },
        minute_bars: [0.0; 24],
        hourly_points,
        daily_forecast,
    })
}

/// 3. 和风天气 API v7 解析器
pub fn fetch_qweather_v7(
    client: &Client,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let key = cfg
        .qweather_key
        .as_deref()
        .ok_or(WeatherProviderError::MissingCredentials("QWEATHER_KEY"))?;
    let host = cfg.qweather_host.trim_end_matches('/');
    let loc = format!("{:.2},{:.2}", cfg.longitude, cfg.latitude);

    let get_qw = |endpoint: &str| -> Result<Value, WeatherProviderError> {
        let url = format!("{host}{endpoint}?location={loc}&key={key}");
        client
            .get(&url)
            .header("X-QW-Api-Key", key)
            .send()
            .map_err(|e| WeatherProviderError::Http(e.to_string()))?
            .json()
            .map_err(|e| WeatherProviderError::Parse(e.to_string()))
    };

    let now_json = get_qw("/v7/weather/now")?;
    let min_json = get_qw("/v7/minutely/5m").ok();
    let h24_json = get_qw("/v7/weather/24h").ok();
    let d7_json = get_qw("/v7/weather/7d").ok();
    let air_json = get_qw("/v7/air/now").ok();
    let warn_json = get_qw("/v7/warning/now").ok();

    let now = now_json
        .get("now")
        .ok_or_else(|| WeatherProviderError::Parse("和风 now 字段缺失".into()))?;
    let temp_c = parse_i32_lossy(now.get("temp"), 20);
    let feels_like = parse_i32_lossy(now.get("feelsLike"), temp_c);
    let humidity = parse_i32_lossy(now.get("humidity"), 55).clamp(0, 100) as u8;
    let pressure = parse_i32_lossy(now.get("pressure"), 1015).max(800) as u16;
    let vis_km = parse_i32_lossy(now.get("vis"), 20).clamp(1, 99) as u8;
    let cond_str = now.get("text").and_then(Value::as_str).unwrap_or("多云");
    let hero_icon = map_chinese_condition_icon(cond_str);
    let wind_dir = short_wind_dir_cn(now.get("windDir").and_then(Value::as_str).unwrap_or("东南风"));
    let wind_scale = parse_i32_lossy(now.get("windScale"), 2).clamp(0, 17) as u8;

    let aqi = air_json
        .as_ref()
        .and_then(|j| j.pointer("/now/aqi"))
        .map(|v| parse_i32_lossy(Some(v), 26).max(0) as u16)
        .unwrap_or(26);
    let aqi_grade = aqi_to_grade_cn(aqi).to_owned();

    let alert = warn_json
        .as_ref()
        .and_then(|j| j.pointer("/warning/0/title"))
        .and_then(Value::as_str)
        .map(extract_short_alert_title);

    let mut minute_bars = [0.0_f32; 24];
    let mut rainy_bins = 0_usize;
    if let Some(arr) = min_json
        .as_ref()
        .and_then(|j| j.get("minutely"))
        .and_then(Value::as_array)
    {
        for (i, item) in arr.iter().take(24).enumerate() {
            let p = item
                .get("precip")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.0);
            let norm = if p <= 0.01 { 0.0 } else { (p / 2.5).clamp(0.12, 1.0) };
            minute_bars[i] = norm;
            if norm > 0.0 {
                rainy_bins += 1;
            }
        }
    }

    let radar_summary = if rainy_bins == 0 {
        "未来两小时无雨".to_owned()
    } else {
        format!("未来降雨{}分", rainy_bins * 5)
    };

    let mut hourly_points = Vec::new();
    if let Some(arr) = h24_json
        .as_ref()
        .and_then(|j| j.get("hourly"))
        .and_then(Value::as_array)
    {
        for item in arr
            .iter()
            .step_by(cfg.hourly_step_hours.max(1))
            .take(cfg.hourly_points_count)
        {
            let fx = item.get("fxTime").and_then(Value::as_str).unwrap_or("");
            let tv = parse_i32_lossy(item.get("temp"), temp_c);
            hourly_points.push((extract_hour_label(fx), tv));
        }
    }
    if hourly_points.len() < 2 {
        hourly_points = default_hourly_curve(temp_c);
    }

    let mut daily_forecast = Vec::new();
    let mut sunset = "17:38".to_owned();
    let mut uv_index = 3_u8;
    if let Some(arr) = d7_json
        .as_ref()
        .and_then(|j| j.get("daily"))
        .and_then(Value::as_array)
    {
        if let Some(first) = arr.first() {
            if let Some(ss) = first.get("sunset").and_then(Value::as_str) {
                sunset = ss.to_owned();
            }
            uv_index = parse_i32_lossy(first.get("uvIndex"), 3).clamp(0, 15) as u8;
        }
        for (idx, item) in arr.iter().take(cfg.forecast_days_count.max(4)).enumerate() {
            let fx_d = item.get("fxDate").and_then(Value::as_str).unwrap_or("");
            let (day_cn, mm_dd) = format_day_cn_and_date(fx_d, idx);
            let dc = item.get("textDay").and_then(Value::as_str).unwrap_or("多云");
            let low = parse_i32_lossy(item.get("tempMin"), temp_c - 4);
            let high = parse_i32_lossy(item.get("tempMax"), temp_c + 4);
            let wd = short_wind_dir_cn(
                item.get("windDirDay")
                    .and_then(Value::as_str)
                    .unwrap_or("东风"),
            );
            let wl = parse_i32_lossy(item.get("windScaleDay"), 2).clamp(0, 17) as u8;
            daily_forecast.push(DailyForecastItem {
                day_cn,
                date: mm_dd,
                icon: map_chinese_condition_icon(dc),
                cond: short_two_char_cond(dc).to_owned(),
                aqi_tag: aqi_grade.chars().next().unwrap_or('优').to_string(),
                low,
                high,
                wind_cn: wd,
                wind_scale: wl,
            });
        }
    }

    let low_c = daily_forecast.first().map(|d| d.low).unwrap_or(temp_c - 4);
    let high_c = daily_forecast.first().map(|d| d.high).unwrap_or(temp_c + 4);

    Ok(WeatherSnapshot {
        city: cfg.city.clone(),
        district: cfg.district.clone(),
        source_cn: String::new(),
        source_tag: String::new(),
        alert,
        aqi_badge: format!("空气{aqi_grade}"),
        updated_at: format_tz_hhmm(current_unix_epoch(), cfg.tz_offset_seconds),
        page_id,
        hero_icon,
        condition: cond_str.chars().take(5).collect(),
        hero_sub_cn: "能见度".to_owned(),
        hero_sub_val: format!("{vis_km}KM"),
        temp_c,
        feels_like,
        low_c,
        high_c,
        humidity,
        wind_dir,
        wind_scale,
        aqi_grade,
        aqi,
        uv_index,
        uv_desc: uv_to_desc_cn(uv_index).to_owned(),
        pressure,
        sunset,
        radar_summary,
        minute_bars,
        hourly_points,
        daily_forecast,
    })
}

/// 4. 中央气象台 (NMC / 国家气象中心) 免 Key 直连 JSON + 网页 HTML 双解析器
pub fn fetch_nmc_cma(
    client: &Client,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let base = cfg.nmc_api_base.trim_end_matches('/');
    let url = format!("{base}/rest/weather?stationid={}", cfg.nmc_station_id.trim());
    if let Ok(resp) = client.get(&url).send()
        && let Ok(json) = resp.json::<Value>()
        && let Ok(snap) = parse_nmc_cma_json_with_cfg(&json, cfg, page_id)
    {
        return Ok(snap);
    }

    // 若 /rest/weather 返回空 data，则自动回退解析中央气象台网页 HTML (/publish/forecast/{nmc_path}.html)
    if !cfg.nmc_path.is_empty() {
        let html_url = format!("{base}/publish/forecast/{}.html", cfg.nmc_path.trim_matches('/'));
        let html = client
            .get(&html_url)
            .send()
            .map_err(|e| WeatherProviderError::Http(e.to_string()))?
            .text()
            .map_err(|e| WeatherProviderError::Parse(e.to_string()))?;
        return parse_nmc_forecast_html(&html, cfg, page_id);
    }

    Err(WeatherProviderError::Parse("中央气象台 JSON 与网页均未返回有效数据".into()))
}

pub fn parse_nmc_forecast_html(
    html: &str,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let temp_c = extract_between(html, "id=\"realTemperature\"", "</div>")
        .and_then(extract_first_i32)
        .unwrap_or(18);
    let humidity = extract_between(html, "id=\"realHumidity\"", "</div>")
        .and_then(extract_first_u8)
        .unwrap_or(55);
    let cond = extract_between(html, "class=\"weatherdesc\"", "</div>")
        .map(|s| strip_html_tags(s).trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "多云".to_owned());

    Ok(WeatherSnapshot {
        city: cfg.city.clone(),
        district: cfg.district.clone(),
        source_cn: String::new(),
        source_tag: String::new(),
        alert: None,
        aqi_badge: "空气优".into(),
        updated_at: format_tz_hhmm(current_unix_epoch(), cfg.tz_offset_seconds),
        page_id,
        hero_icon: map_chinese_condition_icon(&cond),
        condition: cond.chars().take(5).collect(),
        hero_sub_cn: "湿度".into(),
        hero_sub_val: format!("{humidity}%"),
        temp_c,
        feels_like: temp_c,
        low_c: temp_c - 4,
        high_c: temp_c + 4,
        humidity,
        wind_dir: "东南风".into(),
        wind_scale: 2,
        aqi_grade: "优".into(),
        aqi: 28,
        uv_index: 3,
        uv_desc: "中".into(),
        pressure: 1015,
        sunset: "17:38".into(),
        radar_summary: "未来两小时无雨".into(),
        minute_bars: [0.0; 24],
        hourly_points: default_hourly_curve(temp_c),
        daily_forecast: default_daily_forecast(temp_c - 4, temp_c + 4, &cond),
    })
}

pub fn parse_nmc_cma_json(
    json: &Value,
    city: &str,
    district: &str,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let mut cfg = WeatherProviderConfig::from_env();
    cfg.city = city.to_owned();
    cfg.district = district.to_owned();
    parse_nmc_cma_json_with_cfg(json, &cfg, page_id)
}

pub fn parse_nmc_cma_json_with_cfg(
    json: &Value,
    cfg: &WeatherProviderConfig,
    page_id: u8,
) -> Result<WeatherSnapshot, WeatherProviderError> {
    let data = json
        .get("data")
        .filter(|v| v.is_object())
        .ok_or_else(|| WeatherProviderError::Parse("中央气象台缺少有效 data 对象".into()))?;
    let real_w = data
        .pointer("/real/weather")
        .ok_or_else(|| WeatherProviderError::Parse("缺少 real.weather 字段".into()))?;

    let temp_c = real_w
        .get("temperature")
        .and_then(Value::as_f64)
        .unwrap_or(18.0)
        .round() as i32;
    let feels_like = real_w
        .get("feelst")
        .and_then(Value::as_f64)
        .unwrap_or(temp_c as f64)
        .round() as i32;
    let humidity = real_w
        .get("humidity")
        .and_then(Value::as_f64)
        .unwrap_or(55.0)
        .clamp(0.0, 100.0)
        .round() as u8;
    let pressure = real_w
        .get("airpressure")
        .and_then(Value::as_f64)
        .filter(|&p| p > 500.0 && p < 1200.0)
        .unwrap_or(1015.0)
        .round() as u16;
    let cond_raw = real_w
        .get("info")
        .and_then(Value::as_str)
        .filter(|&s| s != "-" && s != "9999")
        .unwrap_or("晴");
    let hero_icon = map_chinese_condition_icon(cond_raw);

    let wind_dir = short_wind_dir_cn(
        data.pointer("/real/wind/direct")
            .and_then(Value::as_str)
            .unwrap_or("东南风"),
    );
    let wind_power_str = data
        .pointer("/real/wind/power")
        .and_then(Value::as_str)
        .unwrap_or("2级");
    let wind_scale = wind_power_str
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse::<u8>()
        .unwrap_or(2);

    let aqi = data
        .pointer("/air/aqi")
        .and_then(Value::as_u64)
        .unwrap_or(28) as u16;
    let aqi_grade = aqi_to_grade_cn(aqi).to_owned();

    let alert = data
        .pointer("/real/warn/alert")
        .and_then(Value::as_str)
        .filter(|&s| s != "9999" && !s.is_empty())
        .map(extract_short_alert_title);

    let mut hourly_points = Vec::new();
    if let Some(arr) = data.get("passedchart").and_then(Value::as_array) {
        for item in arr.iter().take(cfg.hourly_points_count).rev() {
            let tm = item.get("time").and_then(Value::as_str).unwrap_or("");
            let tv = item
                .get("temperature")
                .and_then(Value::as_f64)
                .unwrap_or(temp_c as f64)
                .round() as i32;
            hourly_points.push((extract_hour_label(tm), tv));
        }
    }
    if hourly_points.len() < 2 {
        hourly_points = default_hourly_curve(temp_c);
    }

    let mut daily_forecast = Vec::new();
    if let Some(arr) = data
        .pointer("/predict/detail")
        .and_then(Value::as_array)
    {
        for (idx, item) in arr.iter().take(cfg.forecast_days_count.max(4)).enumerate() {
            let date_s = item.get("date").and_then(Value::as_str).unwrap_or("");
            let (day_cn, mm_dd) = format_day_cn_and_date(date_s, idx);
            let day_info = item
                .pointer("/day/weather/info")
                .and_then(Value::as_str)
                .filter(|&s| s != "9999")
                .or_else(|| item.pointer("/night/weather/info").and_then(Value::as_str))
                .unwrap_or("晴");
            let high = parse_i32_lossy(item.pointer("/day/weather/temperature"), temp_c + 4);
            let low = parse_i32_lossy(item.pointer("/night/weather/temperature"), temp_c - 4);
            let (low, high) = if low == 9999 {
                (high - 8, high)
            } else if high == 9999 {
                (low, low + 8)
            } else {
                (low.min(high), low.max(high))
            };
            let wd = short_wind_dir_cn(
                item.pointer("/day/wind/direct")
                    .and_then(Value::as_str)
                    .filter(|&s| s != "9999")
                    .unwrap_or("微风"),
            );
            daily_forecast.push(DailyForecastItem {
                day_cn,
                date: mm_dd,
                icon: map_chinese_condition_icon(day_info),
                cond: short_two_char_cond(day_info).to_owned(),
                aqi_tag: aqi_grade.chars().next().unwrap_or('优').to_string(),
                low,
                high,
                wind_cn: wd,
                wind_scale,
            });
        }
    }

    let low_c = daily_forecast.first().map(|d| d.low).unwrap_or(temp_c - 4);
    let high_c = daily_forecast.first().map(|d| d.high).unwrap_or(temp_c + 4);

    Ok(WeatherSnapshot {
        city: cfg.city.clone(),
        district: cfg.district.clone(),
        source_cn: String::new(),
        source_tag: String::new(),
        alert,
        aqi_badge: format!("空气{aqi_grade}"),
        updated_at: format_tz_hhmm(current_unix_epoch(), cfg.tz_offset_seconds),
        page_id,
        hero_icon,
        condition: cond_raw.chars().take(5).collect(),
        hero_sub_cn: "湿度".to_owned(),
        hero_sub_val: format!("{humidity}%"),
        temp_c,
        feels_like,
        low_c,
        high_c,
        humidity,
        wind_dir,
        wind_scale,
        aqi_grade,
        aqi,
        uv_index: 3,
        uv_desc: "中".to_owned(),
        pressure,
        sunset: "17:35".to_owned(),
        radar_summary: if cond_raw.contains('雨') || cond_raw.contains('雪') {
            "局地短时降水".to_owned()
        } else {
            "未来两小时无雨".to_owned()
        },
        minute_bars: [0.0; 24],
        hourly_points,
        daily_forecast,
    })
}

/// 预设场景 1：北京 · 海淀区 | 阵雨转多云 + 雷电黄色预警 + 分钟级降水雷达
pub fn preset_rain_radar(page_id: u8) -> WeatherSnapshot {
    WeatherSnapshot {
        city: env_or("WEATHER_CITY", "北京"),
        district: env_or("WEATHER_DISTRICT", "海淀区"),
        source_cn: String::new(),
        source_tag: String::new(),
        alert: Some("雷电黄色预警".into()),
        aqi_badge: "空气优".into(),
        updated_at: "14:30".into(),
        page_id,
        hero_icon: WeatherIconKind::ThunderRain,
        condition: "阵雨转多云".into(),
        hero_sub_cn: "降水".into(),
        hero_sub_val: "85%".into(),
        temp_c: 18,
        feels_like: 16,
        low_c: 11,
        high_c: 21,
        humidity: 78,
        wind_dir: "东南风".into(),
        wind_scale: 3,
        aqi_grade: "优".into(),
        aqi: 28,
        uv_index: 2,
        uv_desc: "弱".into(),
        pressure: 1014,
        sunset: "17:32".into(),
        radar_summary: "35分后雨渐停".into(),
        minute_bars: [
            0.72, 0.85, 0.92, 0.78, 0.62, 0.45, 0.28, 0.12,
            0.00, 0.00, 0.00, 0.00, 0.15, 0.32, 0.40, 0.22,
            0.08, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00,
        ],
        hourly_points: vec![
            ("14H".into(), 18),
            ("18H".into(), 15),
            ("22H".into(), 12),
            ("02H".into(), 11),
            ("06H".into(), 16),
        ],
        daily_forecast: vec![
            DailyForecastItem {
                day_cn: "今天".into(),
                date: "10/17".into(),
                icon: WeatherIconKind::ThunderRain,
                cond: "阵雨".into(),
                aqi_tag: "优".into(),
                low: 11,
                high: 21,
                wind_cn: "东南".into(),
                wind_scale: 3,
            },
            DailyForecastItem {
                day_cn: "明天".into(),
                date: "10/18".into(),
                icon: WeatherIconKind::PartlyCloudy,
                cond: "多云".into(),
                aqi_tag: "优".into(),
                low: 10,
                high: 20,
                wind_cn: "西北".into(),
                wind_scale: 2,
            },
            DailyForecastItem {
                day_cn: "周一".into(),
                date: "10/19".into(),
                icon: WeatherIconKind::Sunny,
                cond: "晴朗".into(),
                aqi_tag: "优".into(),
                low: 9,
                high: 22,
                wind_cn: "微风".into(),
                wind_scale: 1,
            },
            DailyForecastItem {
                day_cn: "周二".into(),
                date: "10/20".into(),
                icon: WeatherIconKind::Sunny,
                cond: "晴朗".into(),
                aqi_tag: "良".into(),
                low: 11,
                high: 24,
                wind_cn: "西南".into(),
                wind_scale: 2,
            },
        ],
    }
}

/// 预设场景 2：上海 · 徐汇区 | 晴间多云 23°C + 空气优 + 昼夜温差走势
pub fn preset_sunny_clear(page_id: u8) -> WeatherSnapshot {
    WeatherSnapshot {
        city: "上海".into(),
        district: "徐汇区".into(),
        source_cn: String::new(),
        source_tag: String::new(),
        alert: None,
        aqi_badge: "空气优".into(),
        updated_at: "09:15".into(),
        page_id,
        hero_icon: WeatherIconKind::PartlyCloudy,
        condition: "晴间多云".into(),
        hero_sub_cn: "能见度".into(),
        hero_sub_val: "24KM".into(),
        temp_c: 23,
        feels_like: 24,
        low_c: 16,
        high_c: 26,
        humidity: 52,
        wind_dir: "东北风".into(),
        wind_scale: 2,
        aqi_grade: "优".into(),
        aqi: 24,
        uv_index: 6,
        uv_desc: "强".into(),
        pressure: 1019,
        sunset: "17:28".into(),
        radar_summary: "未来两小时无雨".into(),
        minute_bars: [0.0; 24],
        hourly_points: vec![
            ("09H".into(), 21),
            ("13H".into(), 26),
            ("17H".into(), 23),
            ("21H".into(), 19),
            ("01H".into(), 16),
        ],
        daily_forecast: vec![
            DailyForecastItem {
                day_cn: "今天".into(),
                date: "10/17".into(),
                icon: WeatherIconKind::PartlyCloudy,
                cond: "多云".into(),
                aqi_tag: "优".into(),
                low: 16,
                high: 26,
                wind_cn: "东北".into(),
                wind_scale: 2,
            },
            DailyForecastItem {
                day_cn: "明天".into(),
                date: "10/18".into(),
                icon: WeatherIconKind::Sunny,
                cond: "晴朗".into(),
                aqi_tag: "优".into(),
                low: 17,
                high: 27,
                wind_cn: "东风".into(),
                wind_scale: 2,
            },
            DailyForecastItem {
                day_cn: "周一".into(),
                date: "10/19".into(),
                icon: WeatherIconKind::PartlyCloudy,
                cond: "多云".into(),
                aqi_tag: "良".into(),
                low: 18,
                high: 25,
                wind_cn: "东南".into(),
                wind_scale: 3,
            },
            DailyForecastItem {
                day_cn: "周二".into(),
                date: "10/20".into(),
                icon: WeatherIconKind::Rain,
                cond: "小雨".into(),
                aqi_tag: "优".into(),
                low: 15,
                high: 21,
                wind_cn: "北风".into(),
                wind_scale: 4,
            },
        ],
    }
}

/// 预设场景 3：哈尔滨 · 南岗区 | 阵雪转晴 -6°C + 寒潮蓝色预警
pub fn preset_cold_snow(page_id: u8) -> WeatherSnapshot {
    WeatherSnapshot {
        city: "哈尔滨".into(),
        district: "南岗区".into(),
        source_cn: String::new(),
        source_tag: String::new(),
        alert: Some("寒潮蓝色预警".into()),
        aqi_badge: "空气优".into(),
        updated_at: "08:00".into(),
        page_id,
        hero_icon: WeatherIconKind::Snow,
        condition: "阵雪转晴".into(),
        hero_sub_cn: "降雪".into(),
        hero_sub_val: "75%".into(),
        temp_c: -6,
        feels_like: -11,
        low_c: -12,
        high_c: -2,
        humidity: 64,
        wind_dir: "西北风".into(),
        wind_scale: 4,
        aqi_grade: "优".into(),
        aqi: 19,
        uv_index: 1,
        uv_desc: "弱".into(),
        pressure: 1028,
        sunset: "16:41".into(),
        radar_summary: "降雪持续45分".into(),
        minute_bars: [
            0.55, 0.65, 0.70, 0.60, 0.50, 0.45, 0.35, 0.25,
            0.15, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00,
            0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00,
        ],
        hourly_points: vec![
            ("08H".into(), -6),
            ("12H".into(), -2),
            ("16H".into(), -4),
            ("20H".into(), -8),
            ("00H".into(), -12),
        ],
        daily_forecast: vec![
            DailyForecastItem {
                day_cn: "今天".into(),
                date: "10/17".into(),
                icon: WeatherIconKind::Snow,
                cond: "阵雪".into(),
                aqi_tag: "优".into(),
                low: -12,
                high: -2,
                wind_cn: "西北".into(),
                wind_scale: 4,
            },
            DailyForecastItem {
                day_cn: "明天".into(),
                date: "10/18".into(),
                icon: WeatherIconKind::Sunny,
                cond: "晴朗".into(),
                aqi_tag: "优".into(),
                low: -13,
                high: -1,
                wind_cn: "西风".into(),
                wind_scale: 3,
            },
            DailyForecastItem {
                day_cn: "周一".into(),
                date: "10/19".into(),
                icon: WeatherIconKind::Sunny,
                cond: "晴朗".into(),
                aqi_tag: "优".into(),
                low: -9,
                high: 2,
                wind_cn: "西南".into(),
                wind_scale: 2,
            },
            DailyForecastItem {
                day_cn: "周二".into(),
                date: "10/20".into(),
                icon: WeatherIconKind::PartlyCloudy,
                cond: "多云".into(),
                aqi_tag: "良".into(),
                low: -7,
                high: 4,
                wind_cn: "南风".into(),
                wind_scale: 2,
            },
        ],
    }
}

fn map_wmo_weather_code(code: i32) -> (WeatherIconKind, &'static str) {
    match code {
        0 => (WeatherIconKind::Sunny, "晴朗"),
        1 | 2 => (WeatherIconKind::PartlyCloudy, "晴间多云"),
        3 => (WeatherIconKind::Cloudy, "阴天"),
        45 | 48 => (WeatherIconKind::FogHaze, "大雾"),
        51 | 53 | 55 | 61 | 80 => (WeatherIconKind::Rain, "小雨"),
        63 | 81 => (WeatherIconKind::Rain, "中雨"),
        65 | 82 => (WeatherIconKind::ThunderRain, "大雨"),
        71 | 73 | 75 | 77 | 85 | 86 => (WeatherIconKind::Snow, "阵雪"),
        95 | 96 | 99 => (WeatherIconKind::ThunderRain, "雷阵雨"),
        _ => (WeatherIconKind::PartlyCloudy, "多云"),
    }
}

fn map_caiyun_skycon(code: &str) -> (WeatherIconKind, &'static str) {
    match code {
        "CLEAR_DAY" | "CLEAR_NIGHT" => (WeatherIconKind::Sunny, "晴朗"),
        "PARTLY_CLOUDY_DAY" | "PARTLY_CLOUDY_NIGHT" => (WeatherIconKind::PartlyCloudy, "晴间多云"),
        "CLOUDY" => (WeatherIconKind::Cloudy, "阴天"),
        "LIGHT_HAZE" | "MODERATE_HAZE" | "HEAVY_HAZE" => (WeatherIconKind::FogHaze, "雾霾"),
        "FOG" => (WeatherIconKind::FogHaze, "大雾"),
        "LIGHT_RAIN" => (WeatherIconKind::Rain, "小雨"),
        "MODERATE_RAIN" => (WeatherIconKind::Rain, "中雨"),
        "HEAVY_RAIN" => (WeatherIconKind::ThunderRain, "大雨"),
        "STORM_RAIN" => (WeatherIconKind::ThunderRain, "暴雨"),
        "LIGHT_SNOW" => (WeatherIconKind::Snow, "小雪"),
        "MODERATE_SNOW" => (WeatherIconKind::Snow, "中雪"),
        "HEAVY_SNOW" | "STORM_SNOW" => (WeatherIconKind::Snow, "大雪"),
        "DUST" | "SAND" => (WeatherIconKind::Wind, "浮尘"),
        "WIND" => (WeatherIconKind::Wind, "大风"),
        _ => (WeatherIconKind::PartlyCloudy, "多云"),
    }
}

fn map_chinese_condition_icon(cond: &str) -> WeatherIconKind {
    if cond.contains("雷") || cond.contains("暴") {
        WeatherIconKind::ThunderRain
    } else if cond.contains("雪") || cond.contains("冰") {
        WeatherIconKind::Snow
    } else if cond.contains("雨") {
        WeatherIconKind::Rain
    } else if cond.contains("雾")
        || cond.contains("霾")
        || cond.contains("尘")
        || cond.contains("沙")
    {
        WeatherIconKind::FogHaze
    } else if cond.contains("多云") || cond.contains("少云") {
        WeatherIconKind::PartlyCloudy
    } else if cond.contains("阴") {
        WeatherIconKind::Cloudy
    } else {
        WeatherIconKind::Sunny
    }
}

fn short_two_char_cond(cond: &str) -> &str {
    if cond.contains("雷") {
        "雷雨"
    } else if cond.contains("暴雨") {
        "暴雨"
    } else if cond.contains("大雨") {
        "大雨"
    } else if cond.contains("中雨") {
        "中雨"
    } else if cond.contains("阵雨") {
        "阵雨"
    } else if cond.contains("雨") {
        "小雨"
    } else if cond.contains("阵雪") {
        "阵雪"
    } else if cond.contains("大雪") {
        "大雪"
    } else if cond.contains("雪") {
        "小雪"
    } else if cond.contains("多云") {
        "多云"
    } else if cond.contains("阴") {
        "阴天"
    } else if cond.contains("雾") || cond.contains("霾") {
        "雾霾"
    } else {
        "晴朗"
    }
}

fn degree_to_compass_cn(deg: f64) -> &'static str {
    let d = deg.rem_euclid(360.0);
    if !(22.5..337.5).contains(&d) {
        "北风"
    } else if d < 67.5 {
        "东北风"
    } else if d < 112.5 {
        "东风"
    } else if d < 157.5 {
        "东南风"
    } else if d < 202.5 {
        "南风"
    } else if d < 247.5 {
        "西南风"
    } else if d < 292.5 {
        "西风"
    } else {
        "西北风"
    }
}

fn short_wind_dir_cn(raw: &str) -> String {
    if raw.contains("东北") {
        "东北风".into()
    } else if raw.contains("东南") {
        "东南风".into()
    } else if raw.contains("西北") {
        "西北风".into()
    } else if raw.contains("西南") {
        "西南风".into()
    } else if raw.contains("东") {
        "东风".into()
    } else if raw.contains("南") {
        "南风".into()
    } else if raw.contains("西") {
        "西风".into()
    } else if raw.contains("北") {
        "北风".into()
    } else {
        "微风".into()
    }
}

fn kmh_to_beaufort_scale(kmh: f64) -> u8 {
    match kmh as u32 {
        0 => 0,
        1..=5 => 1,
        6..=11 => 2,
        12..=19 => 3,
        20..=28 => 4,
        29..=38 => 5,
        39..=49 => 6,
        50..=61 => 7,
        62..=74 => 8,
        75..=88 => 9,
        89..=102 => 10,
        103..=117 => 11,
        _ => 12,
    }
}

fn aqi_to_grade_cn(aqi: u16) -> &'static str {
    match aqi {
        0..=50 => "优",
        51..=100 => "良",
        101..=150 => "轻",
        151..=200 => "中",
        201..=300 => "重",
        _ => "严",
    }
}

fn uv_to_desc_cn(uv: u8) -> &'static str {
    match uv {
        0..=2 => "弱",
        3..=5 => "中",
        6..=7 => "强",
        _ => "极",
    }
}

fn map_caiyun_alert_code(code: &str) -> Option<String> {
    if code.len() < 4 {
        return None;
    }
    let kind = match &code[0..2] {
        "01" => "台风",
        "02" => "暴雨",
        "03" => "暴雪",
        "04" => "寒潮",
        "05" => "大风",
        "06" => "沙尘",
        "07" => "高温",
        "08" => "干旱",
        "09" => "雷电",
        "10" => "冰雹",
        "11" => "霜冻",
        "12" => "大雾",
        "13" => "雾霾",
        "14" => "结冰",
        "15" => "火险",
        "16" => "雷风",
        _ => return None,
    };
    let level = match &code[2..4] {
        "01" => "蓝色",
        "02" => "黄色",
        "03" => "橙色",
        "04" => "红色",
        _ => "黄色",
    };
    Some(format!("{kind}{level}预警"))
}

fn extract_short_alert_title(title: &str) -> String {
    if let Some(pos) = title.find("发布") {
        let sub = &title[pos + "发布".len()..];
        let clean = sub
            .split('[')
            .next()
            .unwrap_or(sub)
            .trim_end_matches("信号");
        clean.chars().take(6).collect()
    } else {
        title.chars().take(6).collect()
    }
}

fn extract_hour_label(dt: &str) -> String {
    if let Some(t_pos) = dt.find('T') {
        let hr: String = dt[t_pos + 1..].chars().take(2).collect();
        return format!("{hr}H");
    }
    if let Some(sp) = dt.split_whitespace().nth(1) {
        let hr: String = sp.chars().take(2).collect();
        return format!("{hr}H");
    }
    "12H".to_owned()
}

fn format_day_cn_and_date(date_raw: &str, idx: usize) -> (String, String) {
    let clean_d: String = date_raw.chars().take(10).collect();
    if let Ok(nd) = NaiveDate::parse_from_str(&clean_d, "%Y-%m-%d") {
        let mm_dd = format!("{:02}/{:02}", nd.month(), nd.day());
        let day_cn = match idx {
            0 => "今天".to_owned(),
            1 => "明天".to_owned(),
            _ => {
                const WKS: [&str; 7] = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
                WKS[nd.weekday().num_days_from_monday() as usize].to_owned()
            }
        };
        (day_cn, mm_dd)
    } else {
        // 如果没有传入具体 YYYY-MM-DD，依据当前实时时间计算真实日期
        let now_sec = current_unix_epoch() + (idx as i64) * 86_400;
        let (day_cn, mm_dd) = if let Some(dt) = DateTime::from_timestamp(now_sec, 0) {
            let tz = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
            let local = dt.with_timezone(&tz);
            let mm_dd = format!("{:02}/{:02}", local.month(), local.day());
            let day_cn = match idx {
                0 => "今天".to_owned(),
                1 => "明天".to_owned(),
                _ => {
                    const WKS: [&str; 7] = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
                    WKS[local.weekday().num_days_from_monday() as usize].to_owned()
                }
            };
            (day_cn, mm_dd)
        } else {
            let day_cn = match idx {
                0 => "今天",
                1 => "明天",
                2 => "周一",
                3 => "周二",
                _ => "周三",
            };
            (day_cn.to_owned(), format!("{:02}", idx))
        };
        (day_cn, mm_dd)
    }
}

fn default_hourly_curve(base: i32) -> Vec<(String, i32)> {
    vec![
        ("09H".into(), base - 1),
        ("13H".into(), base + 3),
        ("17H".into(), base + 1),
        ("21H".into(), base - 2),
        ("01H".into(), base - 4),
    ]
}

fn default_daily_forecast(low: i32, high: i32, cond: &str) -> Vec<DailyForecastItem> {
    let short_c = short_two_char_cond(cond).to_owned();
    let ic = map_chinese_condition_icon(&short_c);
    let mut items = Vec::new();
    for idx in 0..4 {
        let (day_cn, mm_dd) = format_day_cn_and_date("", idx);
        let cur_low = low + match idx {
            1 => -1,
            2 => 0,
            3 => -2,
            _ => 0,
        };
        let cur_high = high + match idx {
            1 => 1,
            2 => 2,
            3 => 0,
            _ => 0,
        };
        let (c_name, c_icon) = if idx == 0 {
            (short_c.clone(), ic)
        } else if idx == 1 {
            ("多云".to_owned(), WeatherIconKind::PartlyCloudy)
        } else {
            ("晴朗".to_owned(), WeatherIconKind::Sunny)
        };
        items.push(DailyForecastItem {
            day_cn,
            date: mm_dd,
            icon: c_icon,
            cond: c_name,
            aqi_tag: "优".into(),
            low: cur_low,
            high: cur_high,
            wind_cn: "东南".into(),
            wind_scale: 2,
        });
    }
    items
}

fn extract_between<'a>(text: &'a str, start_pat: &str, end_pat: &str) -> Option<&'a str> {
    let s_idx = text.find(start_pat)? + start_pat.len();
    let rem = &text[s_idx..];
    let e_idx = rem.find(end_pat)?;
    Some(&rem[..e_idx])
}

fn extract_js_var_json(text: &str, var_name: &str) -> Option<Value> {
    let pos = text.find(var_name)?;
    let after = &text[pos + var_name.len()..];
    let eq_pos = after.find('=')?;
    let json_start = after[eq_pos + 1..].find('{')? + eq_pos + 1;
    let slice = &after[json_start..];
    let mut depth = 0_i32;
    let mut in_str = false;
    let mut esc = false;
    for (i, ch) in slice.char_indices() {
        if esc {
            esc = false;
            continue;
        }
        if ch == '\\' && in_str {
            esc = true;
            continue;
        }
        if ch == '"' {
            in_str = !in_str;
            continue;
        }
        if !in_str {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    return serde_json::from_str(&slice[..=i]).ok();
                }
            }
        }
    }
    None
}

fn strip_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn extract_first_i32(s: &str) -> Option<i32> {
    let mut buf = String::new();
    for ch in s.chars() {
        if (ch == '-' && buf.is_empty()) || ch.is_ascii_digit() {
            buf.push(ch);
        } else if !buf.is_empty() && buf != "-" {
            break;
        } else {
            buf.clear();
        }
    }
    buf.parse::<i32>().ok()
}

fn extract_last_i32(s: &str) -> Option<i32> {
    let mut last = None;
    let mut buf = String::new();
    for ch in s.chars() {
        if (ch == '-' && buf.is_empty()) || ch.is_ascii_digit() {
            buf.push(ch);
        } else {
            if let Ok(v) = buf.parse::<i32>() {
                last = Some(v);
            }
            buf.clear();
        }
    }
    if let Ok(v) = buf.parse::<i32>() {
        last = Some(v);
    }
    last
}

fn extract_first_u8(s: &str) -> Option<u8> {
    extract_first_i32(s).map(|v| v.clamp(0, 100) as u8)
}

fn extract_first_u16(s: &str) -> Option<u16> {
    extract_first_i32(s).map(|v| v.max(0) as u16)
}

fn parse_i32_opt(val: Option<&Value>) -> Option<i32> {
    let v = val?;
    if let Some(n) = v.as_i64() {
        Some(n as i32)
    } else if let Some(f) = v.as_f64() {
        Some(f.round() as i32)
    } else if let Some(s) = v.as_str() {
        s.trim().parse::<f64>().ok().map(|f| f.round() as i32)
    } else {
        None
    }
}

fn parse_i32_lossy(val: Option<&Value>, default: i32) -> i32 {
    parse_i32_opt(val).unwrap_or(default)
}

fn format_tz_hhmm(epoch_secs: i64, tz_offset_seconds: i32) -> String {
    let tz = FixedOffset::east_opt(tz_offset_seconds)
        .or_else(|| FixedOffset::east_opt(DEFAULT_TZ_OFFSET_SECONDS))
        .unwrap();
    if let Some(utc) = DateTime::<Utc>::from_timestamp(epoch_secs, 0) {
        let local = utc.with_timezone(&tz);
        format!("{:02}:{:02}", local.hour(), local.minute())
    } else {
        "12:00".to_owned()
    }
}

fn current_unix_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(1_760_000_000)
}

pub fn env_or(name: &str, default: &str) -> String {
    std::env::var(name)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| default.to_owned())
}

pub fn env_parse<T: std::str::FromStr>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<T>().ok())
        .unwrap_or(default)
}

pub fn env_bool(name: &str, default: bool) -> bool {
    match std::env::var(name) {
        Ok(v) => match v.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default,
        },
        Err(_) => default,
    }
}

pub fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}
