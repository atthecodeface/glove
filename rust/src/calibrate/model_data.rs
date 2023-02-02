//a Noughts and crosses
//pc NOUGHTS_AND_CROSSES_MODEL_JSON
/// Noughts and crosses at 45mm
pub const NOUGHTS_AND_CROSSES_MODEL_JSON: &str = r#"
[
  ["0 bl game",[1.0,0.0,2.0]],
  ["ml game",[1.0,0.0,31.0]],
  ["1 tl game",[1.0,0.0,93.0]],
  ["2 tr game",[108.0,0.0,90.0]],
  ["3 br game",[108.0,0.0,2.0]],
  ["4 bl text",[0.0,106.0,2.0]],
  ["5 tl text",[0.0,106.0,92.0]],
  ["& mid-tip text",[0.0,35.5,76.0]],
  ["& mid-tip game",[72.0,0.0,76.0]],
  ["6cm ruler",[0.0,-9.0,2.0]],
  ["7cm ruler",[0.0,-19.0,2.0]],
  ["8cm ruler",[0.0,-29.0,2.0]],
  ["9cm ruler",[0.0,-39.0,2.0]],
  ["10cm ruler",[0.0,-49.0,2.0]],
  ["text star point above E",[0.0,-49.0,2.0]]
]
"#;

//pc NAC_4V3A6040_JSON
// 2 tr game Wants to be about 1225
// ("4 bl text", 1665, 3093), // Wants to be 1611 3039
// ("5 tl text", 1538, 1194), // Wants to be 1557 1256
// 10cm ruler Wants to be about 4062, 4164
// ("text star point above E", 2824, 2751),
pub const NAC_4V3A6040_JSON: &str = r#"
[
    ["0 bl game", [3184.0, 3752.0]],
    ["ml game", [3194.0, 3072.0]],
    ["& mid-tip game", [4430.0, 1654.0]],
    ["& mid-tip text", [2544.0, 1795.0]],
    ["1 tl game", [3157.0, 1555.0]],
    ["2 tr game", [4955.0, 1200.0]],
    ["3 br game", [4881.0, 3060.0]],
    ["10cm ruler", [4061.0, 4164.0]]
]
"#;
pub const NAC_4V3A6040: &[(&str, isize, isize)] = &[
    ("0 bl game", 3184, 3752),
    ("ml game", 3194, 3072),
    ("& mid-tip game", 4430, 1654),
    ("& mid-tip text", 2544, 1795),
    ("1 tl game", 3157, 1555),
    ("2 tr game", 4955, 1200),
    ("3 br game", 4881, 3060),
    ("10cm ruler", 4061, 4164),
];

//pc NAC_4V3A6041_JSON
// ("text star point above E", 2937, 2881),
// ("text star point above E", 2937, 2881),
pub const NAC_4V3A6041_JSON: &str = r#"
[
    ["0 bl game", [3115.0, 4024.0]],
    ["1 tl game", [3131.0, 1921.0]],
    ["ml game", [3131.0, 3424.0]],
    ["& mid-tip game", [5210.0, 2024.0]],
    ["& mid-tip text", [2822.0, 1751.0]],
    ["2 tr game", [6192.0, 1513.0]],
    ["3 br game", [5807.0, 3487.0]],
    ["5 tl text", [2399.0, 511.0]],
    ["4 bl text", [2436.0, 2381.0]],
    ["7cm ruler", [3252.0, 4421.0]]
]
"#;
pub const NAC_4V3A6041: &[(&str, isize, isize)] = &[
    ("0 bl game", 3115, 4024),
    ("1 tl game", 3131, 1921),
    ("ml game", 3131, 3424),
    ("& mid-tip game", 5210, 2024),
    ("& mid-tip text", 2822, 1751),
    ("2 tr game", 6192, 1513),
    ("3 br game", 5807, 3487),
    ("4 tl text", 2399, 511),
    ("5 bl text", 2436, 2381),
    ("7cm ruler", 3252, 4421),
    // ("text star point above E", 2937, 2881),
    // ("text star point above E", 2937, 2881),
];

//pc NAC_4V3A6042_JSON
// Camera estimate -264.70,-96.50,264.90
// ("2 tr game", 4163, 392),
// ("3 br game",  further up than it seems
// ("text star point above E", 3661, 3217),
// ("5 tl text", 911, 2432),
pub const NAC_4V3A6042_JSON: &str = r#"
[
  ["0 bl game",[4476.0,3913.0]],
  ["1 tl game",[3741.0,1899.0]],
  ["ml game",[4266.0,3313.0]],
  ["& mid-tip game",[4174.0,1143.0]],
  ["& mid-tip text",[2869.0,2466.0]],
  ["4 bl text",[1868.0,4196.0]],
  ["6cm ruler",[4732.0,3911.0]],
  ["7cm ruler",[5003.0,3876.0]],
  ["8cm ruler",[5277.0,3843.0]],
  ["9cm ruler",[5555.0,3807.0]]
]
"#;
pub const NAC_4V3A6042: &[(&str, isize, isize)] = &[
    ("0 bl game", 4476, 3913),
    ("1 tl game", 3741, 1899),
    ("ml game", 4266, 3313),
    ("& mid-tip game", 4174, 1143),
    ("& mid-tip text", 2869, 2466),
    ("4 bl text", 1868, 4196),
    ("6cm ruler", 4732, 3911),
    ("7cm ruler", 5003, 3876),
    ("8cm ruler", 5277, 3843),
    ("9cm ruler", 5555, 3807),
];

//pc N_AND_X_TEST_INF
pub const N_AND_X_TEST_INF: &[(&str, [f64; 3])] = &[
    ("0", [0., 0., 0.]),
    ("1", [108., 0., 0.]),
    ("2", [0., 109., 0.]),
    ("3", [0., 0., 92.]),
    ("4", [108., 0., 92.]),
    ("5", [0., 109., 92.]),
    ("6", [108., 109., 0.]),
];
pub const N_AND_X_TEST_INF_DATA: &[(&str, isize, isize)] = &[
    ("0", 3259, 2330),
    ("1", 4854, 1646),
    ("2", 2375, 1182),
    ("3", 3257, 3331),
    ("4", 4675, 2659),
    ("5", 2447, 2219),
    ("6", 3877, 646),
];
pub const C50MM_STI_POLY: &[f64] = &[
    8.283213378490473e-5,
    1.0010373675395385,
    -0.27346884785220027,
    3.037436155602336,
    -13.196169488132,
    26.7261453717947,
    -19.588972344994545,
];
pub const C50MM_ITS_POLY: &[f64] = &[
    -7.074450991240155e-5,
    0.9983717333234381,
    0.2834468421060592,
    -3.112550737336278,
    13.483235448598862,
    -27.340132132172585,
    20.28454799950123,
];

//a Old
/// Y axis for Y = -2, -1, 0, 1, 2
pub const MODEL_ORIGIN: [f64; 3] = [0., 0., 0.];
pub const MODEL_Y_AXIS: [[f64; 3]; 4] = [
    [0., -100., 0.],
    [0., -50., 0.],
    [0., 50., 0.],
    [0., 100., 0.],
];

/// X axis for X = -1, 0, 1, 2
pub const MODEL_X_AXIS: [[f64; 3]; 3] = [[-100., 0., 0.], [100., 0., 0.], [200., 0., 0.]];

pub const C0_DATA_ALL: [([f64; 3], [f64; 2]); 8] = [
    (MODEL_Y_AXIS[0], [374.591667, 300.550000]),
    (MODEL_Y_AXIS[1], [374.120000, 224.720000]),
    (MODEL_ORIGIN, [375.580000, 156.230000]),
    (MODEL_Y_AXIS[2], [375.598592, 86.098592]),
    (MODEL_Y_AXIS[3], [375.085366, 21.048780]),
    (MODEL_X_AXIS[0], [231.333333, 129.294118]),
    // (MODEL_ORIGIN[2], [375.580000, 156.230000]),
    (MODEL_X_AXIS[1], [504.053398, 175.679612]),
    (MODEL_X_AXIS[2], [619.271084, 195.301205]),
];

pub const C0_DATA: [([f64; 3], [f64; 2]); 4] = [
    (MODEL_Y_AXIS[0], [374.591667, 300.550000]),
    (MODEL_Y_AXIS[3], [375.085366, 21.048780]),
    (MODEL_X_AXIS[0], [231.333333, 129.294118]),
    (MODEL_X_AXIS[2], [619.271084, 195.301205]),
];

// from callibrate/log1.txt line 31
// (1,      1726123,[(0,424.000000,91.000000);(0,321.700000,91.960000);(0,318.175497,92.271523);(0,238.000000,94.000000);(0,239.000000,120.500000);(0,308.706897,192.000000);(0,309.320000,242.380000);(0,408.342105,243.631579);(0,502.091837,290.224490);(0,407.583333,291.666667);(0,206.767857,294.571429);(0,309.311594,293.362319);(0,308.079545,339.965909);(0,308.738095,387.873016);];

// 424.000000,91.000000
// 321.700000,91.960000
// 318.175497,92.271523
// 238.000000,94.000000
// 239.000000,120.500000
// 308.706897,192.000000
// 309.320000,242.380000
// 408.342105,243.631579
// 502.091837,290.224490
// 407.583333,291.666667
// 206.767857,294.571429
// 309.311594,293.362319
// 308.079545,339.965909
// 308.738095,387.873016

// Sorted by X
// 206.767857,294.571429
// 238.000000,94.000000
// 239.000000,120.500000
// 308.706897,192.000000
// 309.320000,242.380000
// 309.311594,293.362319
// 308.079545,339.965909
// 308.738095,387.873016
// 318.175497,92.271523
// 321.700000,91.960000
// 502.091837,290.224490
// 407.583333,291.666667
// 408.342105,243.631579
// 424.000000,91.000000

// Extract Y axis
// 308.706897,192.000000
// 309.320000,242.380000
// 309.311594,293.362319 (origin)
// 308.079545,339.965909
// 308.738095,387.873016
// others
// 206.767857,294.571429
// 238.000000,94.000000
// 239.000000,120.500000
// 318.175497,92.271523
// 321.700000,91.960000
// 502.091837,290.224490
// 407.583333,291.666667
// 408.342105,243.631579
// 424.000000,91.000000

// Others sorted by Y
// 424.000000,91.000000
// 321.700000,91.960000
// 318.175497,92.271523
// 238.000000,94.000000
// 239.000000,120.500000
// 502.091837,290.224490
// 407.583333,291.666667
// 408.342105,243.631579
// 206.767857,294.571429

// Extract X axis - they have same Y as middle Y (origin)
// 502.091837,290.224490
// 407.583333,291.666667
// 206.767857,294.571429

// Resolve data
// Origin:
// 309.311594,293.362319
// Y:
// 308.706897,192.000000
// 309.320000,242.380000
// 308.079545,339.965909
// 308.738095,387.873016
// X:
// 206.767857,294.571429
// 407.583333,291.666667
// 502.091837,290.224490

pub const C1_DATA_ALL: [([f64; 3], [f64; 2]); 8] = [
    (MODEL_Y_AXIS[0], [308.738095, 387.873016]),
    (MODEL_Y_AXIS[1], [308.079545, 339.965909]),
    (MODEL_ORIGIN, [309.311594, 293.362319]),
    (MODEL_Y_AXIS[2], [309.320000, 242.380000]),
    (MODEL_Y_AXIS[3], [308.706897, 192.000000]),
    (MODEL_X_AXIS[0], [206.767857, 294.571429]),
    // (MODEL_ORIGIN[2], [309.311594, 293.362319]),
    (MODEL_X_AXIS[1], [407.583333, 291.666667]),
    (MODEL_X_AXIS[2], [502.091837, 290.224490]),
];
