use signum::{
    chsets::editor::{EChar, ESet, ECHAR_NULL},
    raster::{DrawPrintErr, Page},
};

#[derive(Copy, Clone)]
pub enum KbCset {
    Selected,
    Graphics,
    Labels,
}

const NP_DRAW_COMMANDS: [(u16, u16, u8, KbCset); 116] = [
    (599, 52, 113, KbCset::Graphics),
    (614, 52, 101, KbCset::Graphics),
    (629, 52, 113, KbCset::Graphics),
    (644, 52, 101, KbCset::Graphics),
    (659, 52, 113, KbCset::Graphics),
    (674, 52, 101, KbCset::Graphics),
    (689, 52, 113, KbCset::Graphics),
    (704, 52, 101, KbCset::Graphics),
    (603, 64, 15, KbCset::Selected),
    (614, 64, 1, KbCset::Selected),
    (633, 64, 16, KbCset::Selected),
    (642, 64, 2, KbCset::Selected),
    (664, 64, 17, KbCset::Selected),
    (672, 64, 3, KbCset::Selected),
    (694, 64, 18, KbCset::Selected),
    (703, 64, 4, KbCset::Selected),
    (599, 74, 114, KbCset::Graphics),
    (614, 74, 122, KbCset::Graphics),
    (629, 74, 114, KbCset::Graphics),
    (644, 74, 122, KbCset::Graphics),
    (659, 74, 114, KbCset::Graphics),
    (674, 74, 122, KbCset::Graphics),
    (689, 74, 114, KbCset::Graphics),
    (704, 74, 122, KbCset::Graphics),
    (599, 84, 113, KbCset::Graphics),
    (614, 84, 101, KbCset::Graphics),
    (629, 84, 113, KbCset::Graphics),
    (644, 84, 101, KbCset::Graphics),
    (659, 84, 113, KbCset::Graphics),
    (674, 84, 101, KbCset::Graphics),
    (689, 84, 113, KbCset::Graphics),
    (704, 84, 101, KbCset::Graphics),
    (604, 96, 26, KbCset::Selected),
    (612, 96, 12, KbCset::Selected),
    (633, 96, 27, KbCset::Selected),
    (644, 96, 13, KbCset::Selected),
    (663, 96, 28, KbCset::Selected),
    (673, 96, 14, KbCset::Selected),
    (694, 96, 30, KbCset::Selected),
    (599, 106, 114, KbCset::Graphics),
    (614, 106, 122, KbCset::Graphics),
    (629, 106, 114, KbCset::Graphics),
    (644, 106, 122, KbCset::Graphics),
    (659, 106, 114, KbCset::Graphics),
    (674, 106, 122, KbCset::Graphics),
    (689, 106, 114, KbCset::Graphics),
    (704, 106, 122, KbCset::Graphics),
    (599, 116, 113, KbCset::Graphics),
    (614, 116, 101, KbCset::Graphics),
    (629, 116, 113, KbCset::Graphics),
    (644, 116, 101, KbCset::Graphics),
    (659, 116, 113, KbCset::Graphics),
    (674, 116, 101, KbCset::Graphics),
    (689, 116, 113, KbCset::Graphics),
    (704, 116, 101, KbCset::Graphics),
    (603, 128, 23, KbCset::Selected),
    (612, 128, 9, KbCset::Selected),
    (633, 128, 24, KbCset::Selected),
    (643, 128, 10, KbCset::Selected),
    (663, 128, 25, KbCset::Selected),
    (672, 128, 11, KbCset::Selected),
    (694, 128, 29, KbCset::Selected),
    (599, 138, 114, KbCset::Graphics),
    (614, 138, 122, KbCset::Graphics),
    (629, 138, 114, KbCset::Graphics),
    (644, 138, 122, KbCset::Graphics),
    (659, 138, 114, KbCset::Graphics),
    (674, 138, 122, KbCset::Graphics),
    (689, 138, 114, KbCset::Graphics),
    (704, 138, 122, KbCset::Graphics),
    (599, 148, 113, KbCset::Graphics),
    (614, 148, 101, KbCset::Graphics),
    (629, 148, 113, KbCset::Graphics),
    (644, 148, 101, KbCset::Graphics),
    (659, 148, 113, KbCset::Graphics),
    (674, 148, 101, KbCset::Graphics),
    (689, 148, 113, KbCset::Graphics),
    (704, 148, 101, KbCset::Graphics),
    (694, 150, 69, KbCset::Labels),
    (698, 150, 110, KbCset::Labels),
    (702, 150, 116, KbCset::Labels),
    (705, 150, 101, KbCset::Labels),
    (709, 150, 114, KbCset::Labels),
    (603, 160, 20, KbCset::Selected),
    (614, 160, 6, KbCset::Selected),
    (633, 160, 21, KbCset::Selected),
    (643, 160, 7, KbCset::Selected),
    (663, 160, 22, KbCset::Selected),
    (673, 160, 8, KbCset::Selected),
    (599, 170, 114, KbCset::Graphics),
    (614, 170, 122, KbCset::Graphics),
    (629, 170, 114, KbCset::Graphics),
    (644, 170, 122, KbCset::Graphics),
    (659, 170, 114, KbCset::Graphics),
    (674, 170, 122, KbCset::Graphics),
    (689, 170, 112, KbCset::Graphics),
    (704, 170, 64, KbCset::Graphics),
    (599, 180, 113, KbCset::Graphics),
    (614, 180, 119, KbCset::Graphics),
    (629, 180, 119, KbCset::Graphics),
    (644, 180, 101, KbCset::Graphics),
    (659, 180, 113, KbCset::Graphics),
    (674, 180, 101, KbCset::Graphics),
    (689, 180, 112, KbCset::Graphics),
    (704, 180, 64, KbCset::Graphics),
    (606, 192, 19, KbCset::Selected),
    (620, 192, 5, KbCset::Selected),
    (664, 192, 31, KbCset::Selected),
    (599, 202, 114, KbCset::Graphics),
    (614, 202, 116, KbCset::Graphics),
    (629, 202, 116, KbCset::Graphics),
    (644, 202, 122, KbCset::Graphics),
    (659, 202, 114, KbCset::Graphics),
    (674, 202, 122, KbCset::Graphics),
    (689, 202, 114, KbCset::Graphics),
    (704, 202, 122, KbCset::Graphics),
];

/// These are the draw commands from the KBANTIK document,
/// used to render a keyboard layout
const KB_DRAW_COMMANDS: [(u16, u16, u8, KbCset); 572] = [
    (0, 52, 113, KbCset::Graphics),
    (5, 52, 69, KbCset::Labels),
    (9, 52, 115, KbCset::Labels),
    (12, 52, 119, KbCset::Graphics),
    (13, 52, 99, KbCset::Labels),
    (22, 52, 101, KbCset::Graphics),
    (37, 52, 113, KbCset::Graphics),
    (52, 52, 119, KbCset::Graphics),
    (59, 52, 101, KbCset::Graphics),
    (74, 52, 113, KbCset::Graphics),
    (86, 52, 119, KbCset::Graphics),
    (96, 52, 101, KbCset::Graphics),
    (111, 52, 113, KbCset::Graphics),
    (123, 52, 119, KbCset::Graphics),
    (133, 52, 101, KbCset::Graphics),
    (148, 52, 113, KbCset::Graphics),
    (160, 52, 119, KbCset::Graphics),
    (170, 52, 101, KbCset::Graphics),
    (185, 52, 113, KbCset::Graphics),
    (197, 52, 119, KbCset::Graphics),
    (207, 52, 101, KbCset::Graphics),
    (222, 52, 113, KbCset::Graphics),
    (234, 52, 119, KbCset::Graphics),
    (244, 52, 101, KbCset::Graphics),
    (259, 52, 113, KbCset::Graphics),
    (271, 52, 119, KbCset::Graphics),
    (281, 52, 101, KbCset::Graphics),
    (296, 52, 113, KbCset::Graphics),
    (308, 52, 119, KbCset::Graphics),
    (318, 52, 101, KbCset::Graphics),
    (333, 52, 113, KbCset::Graphics),
    (345, 52, 119, KbCset::Graphics),
    (355, 52, 101, KbCset::Graphics),
    (370, 52, 113, KbCset::Graphics),
    (382, 52, 119, KbCset::Graphics),
    (392, 52, 101, KbCset::Graphics),
    (407, 52, 113, KbCset::Graphics),
    (419, 52, 119, KbCset::Graphics),
    (429, 52, 101, KbCset::Graphics),
    (444, 52, 113, KbCset::Graphics),
    (456, 52, 119, KbCset::Graphics),
    (466, 52, 101, KbCset::Graphics),
    (481, 52, 113, KbCset::Graphics),
    (493, 52, 119, KbCset::Graphics),
    (503, 52, 101, KbCset::Graphics),
    (518, 52, 113, KbCset::Graphics),
    (523, 52, 66, KbCset::Labels),
    (527, 52, 97, KbCset::Labels),
    (531, 52, 99, KbCset::Labels),
    (531, 52, 119, KbCset::Graphics),
    (535, 52, 107, KbCset::Labels),
    (539, 52, 115, KbCset::Labels),
    (540, 52, 119, KbCset::Graphics),
    (543, 52, 112, KbCset::Labels),
    (547, 52, 97, KbCset::Labels),
    (551, 52, 99, KbCset::Labels),
    (555, 52, 101, KbCset::Labels),
    (555, 52, 101, KbCset::Graphics),
    (44, 64, 49, KbCset::Selected),
    (57, 64, 33, KbCset::Selected),
    (81, 64, 50, KbCset::Selected),
    (98, 64, 34, KbCset::Selected),
    (118, 64, 51, KbCset::Selected),
    (131, 64, 32, KbCset::Selected),
    (155, 64, 52, KbCset::Selected),
    (169, 64, 36, KbCset::Selected),
    (192, 64, 53, KbCset::Selected),
    (204, 64, 37, KbCset::Selected),
    (229, 64, 54, KbCset::Selected),
    (239, 64, 38, KbCset::Selected),
    (266, 64, 55, KbCset::Selected),
    (276, 64, 47, KbCset::Selected),
    (303, 64, 56, KbCset::Selected),
    (313, 64, 40, KbCset::Selected),
    (340, 64, 57, KbCset::Selected),
    (355, 64, 41, KbCset::Selected),
    (377, 64, 48, KbCset::Selected),
    (390, 64, 61, KbCset::Selected),
    (414, 64, 127, KbCset::Selected),
    (428, 64, 63, KbCset::Selected),
    (451, 64, 39, KbCset::Selected),
    (461, 64, 96, KbCset::Selected),
    (492, 64, 35, KbCset::Selected),
    (503, 64, 94, KbCset::Selected),
    (0, 74, 114, KbCset::Graphics),
    (12, 74, 116, KbCset::Graphics),
    (22, 74, 122, KbCset::Graphics),
    (37, 74, 114, KbCset::Graphics),
    (49, 74, 116, KbCset::Graphics),
    (59, 74, 122, KbCset::Graphics),
    (74, 74, 114, KbCset::Graphics),
    (86, 74, 116, KbCset::Graphics),
    (96, 74, 122, KbCset::Graphics),
    (111, 74, 114, KbCset::Graphics),
    (123, 74, 116, KbCset::Graphics),
    (133, 74, 122, KbCset::Graphics),
    (148, 74, 114, KbCset::Graphics),
    (160, 74, 116, KbCset::Graphics),
    (170, 74, 122, KbCset::Graphics),
    (185, 74, 114, KbCset::Graphics),
    (197, 74, 116, KbCset::Graphics),
    (207, 74, 122, KbCset::Graphics),
    (222, 74, 114, KbCset::Graphics),
    (234, 74, 116, KbCset::Graphics),
    (244, 74, 122, KbCset::Graphics),
    (259, 74, 114, KbCset::Graphics),
    (271, 74, 116, KbCset::Graphics),
    (281, 74, 122, KbCset::Graphics),
    (296, 74, 114, KbCset::Graphics),
    (308, 74, 116, KbCset::Graphics),
    (318, 74, 122, KbCset::Graphics),
    (333, 74, 114, KbCset::Graphics),
    (345, 74, 116, KbCset::Graphics),
    (355, 74, 122, KbCset::Graphics),
    (370, 74, 114, KbCset::Graphics),
    (382, 74, 116, KbCset::Graphics),
    (392, 74, 122, KbCset::Graphics),
    (407, 74, 114, KbCset::Graphics),
    (419, 74, 116, KbCset::Graphics),
    (429, 74, 122, KbCset::Graphics),
    (444, 74, 114, KbCset::Graphics),
    (456, 74, 116, KbCset::Graphics),
    (466, 74, 122, KbCset::Graphics),
    (481, 74, 114, KbCset::Graphics),
    (493, 74, 116, KbCset::Graphics),
    (503, 74, 122, KbCset::Graphics),
    (518, 74, 114, KbCset::Graphics),
    (530, 74, 116, KbCset::Graphics),
    (540, 74, 116, KbCset::Graphics),
    (555, 74, 122, KbCset::Graphics),
    (0, 84, 113, KbCset::Graphics),
    (5, 84, 84, KbCset::Labels),
    (9, 84, 97, KbCset::Labels),
    (12, 84, 119, KbCset::Graphics),
    (13, 84, 98, KbCset::Labels),
    (22, 84, 119, KbCset::Graphics),
    (37, 84, 101, KbCset::Graphics),
    (52, 84, 113, KbCset::Graphics),
    (67, 84, 119, KbCset::Graphics),
    (74, 84, 101, KbCset::Graphics),
    (89, 84, 113, KbCset::Graphics),
    (101, 84, 119, KbCset::Graphics),
    (111, 84, 101, KbCset::Graphics),
    (126, 84, 113, KbCset::Graphics),
    (138, 84, 119, KbCset::Graphics),
    (148, 84, 101, KbCset::Graphics),
    (163, 84, 113, KbCset::Graphics),
    (175, 84, 119, KbCset::Graphics),
    (185, 84, 101, KbCset::Graphics),
    (200, 84, 113, KbCset::Graphics),
    (212, 84, 119, KbCset::Graphics),
    (222, 84, 101, KbCset::Graphics),
    (237, 84, 113, KbCset::Graphics),
    (249, 84, 119, KbCset::Graphics),
    (259, 84, 101, KbCset::Graphics),
    (274, 84, 113, KbCset::Graphics),
    (286, 84, 119, KbCset::Graphics),
    (296, 84, 101, KbCset::Graphics),
    (311, 84, 113, KbCset::Graphics),
    (323, 84, 119, KbCset::Graphics),
    (333, 84, 101, KbCset::Graphics),
    (348, 84, 113, KbCset::Graphics),
    (360, 84, 119, KbCset::Graphics),
    (370, 84, 101, KbCset::Graphics),
    (385, 84, 113, KbCset::Graphics),
    (397, 84, 119, KbCset::Graphics),
    (407, 84, 101, KbCset::Graphics),
    (422, 84, 113, KbCset::Graphics),
    (434, 84, 119, KbCset::Graphics),
    (444, 84, 101, KbCset::Graphics),
    (459, 84, 113, KbCset::Graphics),
    (471, 84, 119, KbCset::Graphics),
    (481, 84, 101, KbCset::Graphics),
    (496, 84, 113, KbCset::Graphics),
    (508, 84, 119, KbCset::Graphics),
    (518, 84, 101, KbCset::Graphics),
    (533, 84, 113, KbCset::Graphics),
    (538, 84, 68, KbCset::Labels),
    (543, 84, 101, KbCset::Labels),
    (546, 84, 119, KbCset::Graphics),
    (547, 84, 108, KbCset::Labels),
    (549, 84, 101, KbCset::Labels),
    (553, 84, 116, KbCset::Labels),
    (555, 84, 101, KbCset::Graphics),
    (556, 84, 101, KbCset::Labels),
    (59, 96, 113, KbCset::Selected),
    (71, 96, 81, KbCset::Selected),
    (94, 96, 119, KbCset::Selected),
    (107, 96, 87, KbCset::Selected),
    (133, 96, 101, KbCset::Selected),
    (145, 96, 69, KbCset::Selected),
    (170, 96, 114, KbCset::Selected),
    (182, 96, 82, KbCset::Selected),
    (207, 96, 116, KbCset::Selected),
    (220, 96, 84, KbCset::Selected),
    (244, 96, 122, KbCset::Selected),
    (256, 96, 90, KbCset::Selected),
    (281, 96, 117, KbCset::Selected),
    (293, 96, 85, KbCset::Selected),
    (318, 96, 105, KbCset::Selected),
    (332, 96, 73, KbCset::Selected),
    (353, 96, 111, KbCset::Selected),
    (368, 96, 79, KbCset::Selected),
    (392, 96, 112, KbCset::Selected),
    (405, 96, 80, KbCset::Selected),
    (429, 96, 64, KbCset::Selected),
    (442, 96, 92, KbCset::Selected),
    (466, 96, 43, KbCset::Selected),
    (478, 96, 42, KbCset::Selected),
    (496, 100, 112, KbCset::Graphics),
    (0, 106, 114, KbCset::Graphics),
    (12, 106, 116, KbCset::Graphics),
    (22, 106, 116, KbCset::Graphics),
    (37, 106, 122, KbCset::Graphics),
    (52, 106, 114, KbCset::Graphics),
    (64, 106, 116, KbCset::Graphics),
    (74, 106, 122, KbCset::Graphics),
    (89, 106, 114, KbCset::Graphics),
    (101, 106, 116, KbCset::Graphics),
    (111, 106, 122, KbCset::Graphics),
    (126, 106, 114, KbCset::Graphics),
    (138, 106, 116, KbCset::Graphics),
    (148, 106, 122, KbCset::Graphics),
    (163, 106, 114, KbCset::Graphics),
    (175, 106, 116, KbCset::Graphics),
    (185, 106, 122, KbCset::Graphics),
    (200, 106, 114, KbCset::Graphics),
    (212, 106, 116, KbCset::Graphics),
    (222, 106, 122, KbCset::Graphics),
    (237, 106, 114, KbCset::Graphics),
    (249, 106, 116, KbCset::Graphics),
    (259, 106, 122, KbCset::Graphics),
    (274, 106, 114, KbCset::Graphics),
    (286, 106, 116, KbCset::Graphics),
    (296, 106, 122, KbCset::Graphics),
    (311, 106, 114, KbCset::Graphics),
    (323, 106, 116, KbCset::Graphics),
    (333, 106, 122, KbCset::Graphics),
    (348, 106, 114, KbCset::Graphics),
    (360, 106, 116, KbCset::Graphics),
    (370, 106, 122, KbCset::Graphics),
    (385, 106, 114, KbCset::Graphics),
    (397, 106, 116, KbCset::Graphics),
    (407, 106, 122, KbCset::Graphics),
    (422, 106, 114, KbCset::Graphics),
    (434, 106, 116, KbCset::Graphics),
    (444, 106, 122, KbCset::Graphics),
    (459, 106, 114, KbCset::Graphics),
    (471, 106, 116, KbCset::Graphics),
    (481, 106, 122, KbCset::Graphics),
    (518, 106, 64, KbCset::Graphics),
    (533, 106, 114, KbCset::Graphics),
    (545, 106, 116, KbCset::Graphics),
    (555, 106, 122, KbCset::Graphics),
    (0, 116, 113, KbCset::Graphics),
    (5, 116, 67, KbCset::Labels),
    (10, 116, 111, KbCset::Labels),
    (12, 116, 119, KbCset::Graphics),
    (14, 116, 110, KbCset::Labels),
    (18, 116, 116, KbCset::Labels),
    (21, 116, 114, KbCset::Labels),
    (22, 116, 119, KbCset::Graphics),
    (24, 116, 111, KbCset::Labels),
    (28, 116, 108, KbCset::Labels),
    (37, 116, 119, KbCset::Graphics),
    (52, 116, 101, KbCset::Graphics),
    (67, 116, 113, KbCset::Graphics),
    (82, 116, 119, KbCset::Graphics),
    (89, 116, 101, KbCset::Graphics),
    (104, 116, 113, KbCset::Graphics),
    (116, 116, 119, KbCset::Graphics),
    (126, 116, 101, KbCset::Graphics),
    (141, 116, 113, KbCset::Graphics),
    (153, 116, 119, KbCset::Graphics),
    (163, 116, 101, KbCset::Graphics),
    (178, 116, 113, KbCset::Graphics),
    (190, 116, 119, KbCset::Graphics),
    (200, 116, 101, KbCset::Graphics),
    (215, 116, 113, KbCset::Graphics),
    (227, 116, 119, KbCset::Graphics),
    (237, 116, 101, KbCset::Graphics),
    (252, 116, 113, KbCset::Graphics),
    (264, 116, 119, KbCset::Graphics),
    (274, 116, 101, KbCset::Graphics),
    (289, 116, 113, KbCset::Graphics),
    (301, 116, 119, KbCset::Graphics),
    (311, 116, 101, KbCset::Graphics),
    (326, 116, 113, KbCset::Graphics),
    (338, 116, 119, KbCset::Graphics),
    (348, 116, 101, KbCset::Graphics),
    (363, 116, 113, KbCset::Graphics),
    (375, 116, 119, KbCset::Graphics),
    (385, 116, 101, KbCset::Graphics),
    (400, 116, 113, KbCset::Graphics),
    (412, 116, 119, KbCset::Graphics),
    (422, 116, 101, KbCset::Graphics),
    (437, 116, 113, KbCset::Graphics),
    (449, 116, 119, KbCset::Graphics),
    (459, 116, 101, KbCset::Graphics),
    (474, 116, 113, KbCset::Graphics),
    (479, 116, 82, KbCset::Labels),
    (483, 116, 101, KbCset::Labels),
    (485, 116, 49, KbCset::Graphics),
    (487, 116, 116, KbCset::Labels),
    (490, 116, 117, KbCset::Labels),
    (494, 116, 114, KbCset::Labels),
    (497, 116, 110, KbCset::Labels),
    (533, 116, 113, KbCset::Graphics),
    (546, 116, 119, KbCset::Graphics),
    (555, 116, 101, KbCset::Graphics),
    (518, 122, 64, KbCset::Graphics),
    (74, 128, 97, KbCset::Selected),
    (88, 128, 65, KbCset::Selected),
    (111, 128, 115, KbCset::Selected),
    (124, 128, 83, KbCset::Selected),
    (148, 128, 100, KbCset::Selected),
    (160, 128, 68, KbCset::Selected),
    (185, 128, 102, KbCset::Selected),
    (198, 128, 70, KbCset::Selected),
    (222, 128, 103, KbCset::Selected),
    (234, 128, 71, KbCset::Selected),
    (259, 128, 104, KbCset::Selected),
    (272, 128, 72, KbCset::Selected),
    (296, 128, 106, KbCset::Selected),
    (309, 128, 74, KbCset::Selected),
    (333, 128, 107, KbCset::Selected),
    (347, 128, 75, KbCset::Selected),
    (370, 128, 108, KbCset::Selected),
    (384, 128, 76, KbCset::Selected),
    (406, 128, 91, KbCset::Selected),
    (419, 128, 123, KbCset::Selected),
    (444, 128, 93, KbCset::Selected),
    (455, 128, 125, KbCset::Selected),
    (541, 128, 126, KbCset::Selected),
    (558, 128, 124, KbCset::Selected),
    (0, 138, 114, KbCset::Graphics),
    (12, 138, 116, KbCset::Graphics),
    (22, 138, 116, KbCset::Graphics),
    (37, 138, 116, KbCset::Graphics),
    (52, 138, 122, KbCset::Graphics),
    (67, 138, 114, KbCset::Graphics),
    (79, 138, 116, KbCset::Graphics),
    (89, 138, 122, KbCset::Graphics),
    (104, 138, 114, KbCset::Graphics),
    (116, 138, 116, KbCset::Graphics),
    (126, 138, 122, KbCset::Graphics),
    (141, 138, 114, KbCset::Graphics),
    (153, 138, 116, KbCset::Graphics),
    (163, 138, 122, KbCset::Graphics),
    (178, 138, 114, KbCset::Graphics),
    (190, 138, 116, KbCset::Graphics),
    (200, 138, 122, KbCset::Graphics),
    (215, 138, 114, KbCset::Graphics),
    (227, 138, 116, KbCset::Graphics),
    (237, 138, 122, KbCset::Graphics),
    (252, 138, 114, KbCset::Graphics),
    (264, 138, 116, KbCset::Graphics),
    (274, 138, 122, KbCset::Graphics),
    (289, 138, 114, KbCset::Graphics),
    (302, 138, 116, KbCset::Graphics),
    (311, 138, 122, KbCset::Graphics),
    (326, 138, 114, KbCset::Graphics),
    (338, 138, 116, KbCset::Graphics),
    (348, 138, 122, KbCset::Graphics),
    (363, 138, 114, KbCset::Graphics),
    (375, 138, 116, KbCset::Graphics),
    (385, 138, 122, KbCset::Graphics),
    (400, 138, 114, KbCset::Graphics),
    (412, 138, 116, KbCset::Graphics),
    (422, 138, 122, KbCset::Graphics),
    (437, 138, 114, KbCset::Graphics),
    (449, 138, 116, KbCset::Graphics),
    (459, 138, 122, KbCset::Graphics),
    (474, 138, 114, KbCset::Graphics),
    (489, 138, 116, KbCset::Graphics),
    (501, 138, 116, KbCset::Graphics),
    (512, 138, 116, KbCset::Graphics),
    (518, 138, 122, KbCset::Graphics),
    (533, 138, 114, KbCset::Graphics),
    (545, 138, 116, KbCset::Graphics),
    (555, 138, 122, KbCset::Graphics),
    (0, 148, 113, KbCset::Graphics),
    (5, 148, 83, KbCset::Labels),
    (9, 148, 104, KbCset::Labels),
    (12, 148, 119, KbCset::Graphics),
    (13, 148, 105, KbCset::Labels),
    (15, 148, 102, KbCset::Labels),
    (17, 148, 119, KbCset::Graphics),
    (18, 148, 116, KbCset::Labels),
    (32, 148, 101, KbCset::Graphics),
    (47, 148, 113, KbCset::Graphics),
    (62, 148, 119, KbCset::Graphics),
    (69, 148, 101, KbCset::Graphics),
    (84, 148, 113, KbCset::Graphics),
    (96, 148, 119, KbCset::Graphics),
    (106, 148, 101, KbCset::Graphics),
    (121, 148, 113, KbCset::Graphics),
    (133, 148, 119, KbCset::Graphics),
    (143, 148, 101, KbCset::Graphics),
    (158, 148, 113, KbCset::Graphics),
    (170, 148, 119, KbCset::Graphics),
    (180, 148, 101, KbCset::Graphics),
    (195, 148, 113, KbCset::Graphics),
    (207, 148, 119, KbCset::Graphics),
    (217, 148, 101, KbCset::Graphics),
    (232, 148, 113, KbCset::Graphics),
    (244, 148, 119, KbCset::Graphics),
    (254, 148, 101, KbCset::Graphics),
    (269, 148, 113, KbCset::Graphics),
    (283, 148, 119, KbCset::Graphics),
    (291, 148, 101, KbCset::Graphics),
    (306, 148, 113, KbCset::Graphics),
    (318, 148, 119, KbCset::Graphics),
    (328, 148, 101, KbCset::Graphics),
    (343, 148, 113, KbCset::Graphics),
    (355, 148, 119, KbCset::Graphics),
    (365, 148, 101, KbCset::Graphics),
    (380, 148, 113, KbCset::Graphics),
    (392, 148, 119, KbCset::Graphics),
    (402, 148, 101, KbCset::Graphics),
    (417, 148, 113, KbCset::Graphics),
    (429, 148, 119, KbCset::Graphics),
    (439, 148, 101, KbCset::Graphics),
    (454, 148, 113, KbCset::Graphics),
    (459, 148, 83, KbCset::Labels),
    (463, 148, 104, KbCset::Labels),
    (466, 148, 119, KbCset::Graphics),
    (467, 148, 105, KbCset::Labels),
    (469, 148, 102, KbCset::Labels),
    (471, 148, 119, KbCset::Graphics),
    (472, 148, 116, KbCset::Labels),
    (486, 148, 101, KbCset::Graphics),
    (54, 160, 60, KbCset::Selected),
    (66, 160, 62, KbCset::Selected),
    (91, 160, 121, KbCset::Selected),
    (103, 160, 89, KbCset::Selected),
    (128, 160, 120, KbCset::Selected),
    (140, 160, 88, KbCset::Selected),
    (165, 160, 99, KbCset::Selected),
    (178, 160, 67, KbCset::Selected),
    (200, 160, 118, KbCset::Selected),
    (214, 160, 86, KbCset::Selected),
    (239, 160, 98, KbCset::Selected),
    (251, 160, 66, KbCset::Selected),
    (276, 160, 110, KbCset::Selected),
    (287, 160, 78, KbCset::Selected),
    (312, 160, 109, KbCset::Selected),
    (324, 160, 77, KbCset::Selected),
    (350, 160, 44, KbCset::Selected),
    (363, 160, 59, KbCset::Selected),
    (387, 160, 46, KbCset::Selected),
    (400, 160, 58, KbCset::Selected),
    (425, 160, 45, KbCset::Selected),
    (438, 160, 95, KbCset::Selected),
    (0, 170, 114, KbCset::Graphics),
    (12, 170, 116, KbCset::Graphics),
    (17, 170, 116, KbCset::Graphics),
    (32, 170, 122, KbCset::Graphics),
    (47, 170, 114, KbCset::Graphics),
    (59, 170, 116, KbCset::Graphics),
    (69, 170, 122, KbCset::Graphics),
    (84, 170, 114, KbCset::Graphics),
    (96, 170, 116, KbCset::Graphics),
    (106, 170, 122, KbCset::Graphics),
    (121, 170, 114, KbCset::Graphics),
    (133, 170, 116, KbCset::Graphics),
    (143, 170, 122, KbCset::Graphics),
    (158, 170, 114, KbCset::Graphics),
    (170, 170, 116, KbCset::Graphics),
    (180, 170, 122, KbCset::Graphics),
    (195, 170, 114, KbCset::Graphics),
    (207, 170, 116, KbCset::Graphics),
    (217, 170, 122, KbCset::Graphics),
    (232, 170, 114, KbCset::Graphics),
    (244, 170, 116, KbCset::Graphics),
    (254, 170, 122, KbCset::Graphics),
    (269, 170, 114, KbCset::Graphics),
    (283, 170, 116, KbCset::Graphics),
    (291, 170, 122, KbCset::Graphics),
    (306, 170, 114, KbCset::Graphics),
    (318, 170, 116, KbCset::Graphics),
    (328, 170, 122, KbCset::Graphics),
    (343, 170, 114, KbCset::Graphics),
    (355, 170, 116, KbCset::Graphics),
    (365, 170, 122, KbCset::Graphics),
    (380, 170, 114, KbCset::Graphics),
    (392, 170, 116, KbCset::Graphics),
    (402, 170, 122, KbCset::Graphics),
    (417, 170, 114, KbCset::Graphics),
    (429, 170, 116, KbCset::Graphics),
    (439, 170, 122, KbCset::Graphics),
    (454, 170, 114, KbCset::Graphics),
    (466, 170, 116, KbCset::Graphics),
    (471, 170, 116, KbCset::Graphics),
    (486, 170, 122, KbCset::Graphics),
    (47, 180, 113, KbCset::Graphics),
    (52, 180, 65, KbCset::Labels),
    (57, 180, 108, KbCset::Labels),
    (59, 180, 116, KbCset::Labels),
    (59, 180, 119, KbCset::Graphics),
    (62, 180, 101, KbCset::Labels),
    (66, 180, 114, KbCset::Labels),
    (69, 180, 110, KbCset::Labels),
    (69, 180, 119, KbCset::Graphics),
    (73, 180, 97, KbCset::Labels),
    (77, 180, 116, KbCset::Labels),
    (80, 180, 101, KbCset::Labels),
    (84, 180, 101, KbCset::Graphics),
    (99, 180, 113, KbCset::Graphics),
    (114, 180, 119, KbCset::Graphics),
    (129, 180, 119, KbCset::Graphics),
    (144, 180, 119, KbCset::Graphics),
    (159, 180, 119, KbCset::Graphics),
    (174, 180, 119, KbCset::Graphics),
    (189, 180, 119, KbCset::Graphics),
    (204, 180, 119, KbCset::Graphics),
    (219, 180, 119, KbCset::Graphics),
    (234, 180, 119, KbCset::Graphics),
    (249, 180, 119, KbCset::Graphics),
    (264, 180, 119, KbCset::Graphics),
    (279, 180, 119, KbCset::Graphics),
    (294, 180, 119, KbCset::Graphics),
    (309, 180, 119, KbCset::Graphics),
    (324, 180, 119, KbCset::Graphics),
    (339, 180, 119, KbCset::Graphics),
    (354, 180, 119, KbCset::Graphics),
    (369, 180, 119, KbCset::Graphics),
    (384, 180, 119, KbCset::Graphics),
    (399, 180, 119, KbCset::Graphics),
    (414, 180, 101, KbCset::Graphics),
    (429, 180, 113, KbCset::Graphics),
    (434, 180, 67, KbCset::Labels),
    (439, 180, 97, KbCset::Labels),
    (441, 180, 119, KbCset::Graphics),
    (443, 180, 112, KbCset::Labels),
    (447, 180, 115, KbCset::Labels),
    (451, 180, 119, KbCset::Graphics),
    (453, 180, 76, KbCset::Labels),
    (457, 180, 111, KbCset::Labels),
    (461, 180, 99, KbCset::Labels),
    (465, 180, 107, KbCset::Labels),
    (466, 180, 101, KbCset::Graphics),
    (47, 202, 114, KbCset::Graphics),
    (59, 202, 116, KbCset::Graphics),
    (69, 202, 116, KbCset::Graphics),
    (84, 202, 122, KbCset::Graphics),
    (99, 202, 114, KbCset::Graphics),
    (114, 202, 116, KbCset::Graphics),
    (129, 202, 116, KbCset::Graphics),
    (144, 202, 116, KbCset::Graphics),
    (159, 202, 116, KbCset::Graphics),
    (174, 202, 116, KbCset::Graphics),
    (189, 202, 116, KbCset::Graphics),
    (204, 202, 116, KbCset::Graphics),
    (219, 202, 116, KbCset::Graphics),
    (234, 202, 116, KbCset::Graphics),
    (249, 202, 116, KbCset::Graphics),
    (264, 202, 116, KbCset::Graphics),
    (279, 202, 116, KbCset::Graphics),
    (294, 202, 116, KbCset::Graphics),
    (309, 202, 116, KbCset::Graphics),
    (324, 202, 116, KbCset::Graphics),
    (339, 202, 116, KbCset::Graphics),
    (354, 202, 116, KbCset::Graphics),
    (369, 202, 116, KbCset::Graphics),
    (384, 202, 116, KbCset::Graphics),
    (399, 202, 116, KbCset::Graphics),
    (414, 202, 122, KbCset::Graphics),
    (429, 202, 114, KbCset::Graphics),
    (441, 202, 116, KbCset::Graphics),
    (451, 202, 116, KbCset::Graphics),
    (466, 202, 122, KbCset::Graphics),
];

fn get_gchar(gchar: u8) -> &'static EChar<'static> {
    match gchar {
        49 => &EChar {
            width: 15,
            height: 10,
            top: 0,
            buf: &[
                0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 255, 242, 255, 242, 0, 2, 255, 254,
            ],
        },
        64 => &EChar {
            width: 15,
            height: 24,
            top: 0,
            buf: &[
                0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18,
                0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18,
            ],
        },
        101 => &EChar {
            width: 15,
            height: 18,
            top: 6,
            buf: &[
                255, 254, 255, 254, 0, 2, 255, 242, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0,
                18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18,
            ],
        },
        112 => &EChar {
            width: 15,
            height: 24,
            top: 0,
            buf: &[
                144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0,
                144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0,
                144, 0, 144, 0, 144, 0, 144, 0,
            ],
        },
        113 => &EChar {
            width: 15,
            height: 18,
            top: 6,
            buf: &[
                255, 254, 255, 254, 128, 0, 159, 254, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144,
                0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0,
            ],
        },
        114 => &EChar {
            width: 15,
            height: 18,
            top: 0,
            buf: &[
                144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0, 144, 0,
                144, 0, 144, 0, 144, 0, 159, 254, 128, 0, 128, 0, 255, 254, 255, 254,
            ],
        },
        116 => &EChar {
            width: 15,
            height: 5,
            top: 13,
            buf: &[255, 254, 0, 0, 0, 0, 255, 254, 255, 254],
        },
        119 => &EChar {
            width: 15,
            height: 4,
            top: 6,
            buf: &[255, 254, 255, 254, 0, 0, 255, 254],
        },
        122 => &EChar {
            width: 15,
            height: 18,
            top: 0,
            buf: &[
                0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18, 0, 18,
                0, 18, 255, 242, 0, 2, 0, 2, 255, 254, 255, 254,
            ],
        },
        _ => &ECHAR_NULL,
    }
}

fn get_lchar(lchar: u8) -> &'static EChar<'static> {
    match lchar {
        65 => &EChar {
            width: 5,
            height: 7,
            top: 11,
            buf: &[32, 0, 32, 0, 80, 0, 80, 0, 112, 0, 136, 0, 136, 0],
        },
        66 => &EChar {
            width: 4,
            height: 7,
            top: 11,
            buf: &[96, 0, 80, 0, 80, 0, 112, 0, 80, 0, 80, 0, 96, 0],
        },
        67 => &EChar {
            width: 5,
            height: 7,
            top: 11,
            buf: &[48, 0, 72, 0, 64, 0, 64, 0, 64, 0, 72, 0, 48, 0],
        },
        68 => &EChar {
            width: 5,
            height: 7,
            top: 11,
            buf: &[112, 0, 72, 0, 72, 0, 72, 0, 72, 0, 72, 0, 112, 0],
        },
        69 => &EChar {
            width: 4,
            height: 7,
            top: 11,
            buf: &[112, 0, 64, 0, 64, 0, 112, 0, 64, 0, 64, 0, 112, 0],
        },
        76 => &EChar {
            width: 4,
            height: 7,
            top: 11,
            buf: &[64, 0, 64, 0, 64, 0, 64, 0, 64, 0, 64, 0, 112, 0],
        },
        82 => &EChar {
            width: 4,
            height: 7,
            top: 11,
            buf: &[96, 0, 80, 0, 80, 0, 96, 0, 96, 0, 80, 0, 80, 0],
        },
        83 => &EChar {
            width: 4,
            height: 7,
            top: 11,
            buf: &[48, 0, 80, 0, 64, 0, 96, 0, 16, 0, 80, 0, 96, 0],
        },
        84 => &EChar {
            width: 4,
            height: 7,
            top: 11,
            buf: &[112, 0, 32, 0, 32, 0, 32, 0, 32, 0, 32, 0, 32, 0],
        },
        97 => &EChar {
            width: 4,
            height: 5,
            top: 13,
            buf: &[96, 0, 16, 0, 112, 0, 80, 0, 112, 0],
        },
        98 => &EChar {
            width: 4,
            height: 7,
            top: 11,
            buf: &[64, 0, 64, 0, 112, 0, 80, 0, 80, 0, 80, 0, 112, 0],
        },
        99 => &EChar {
            width: 4,
            height: 5,
            top: 13,
            buf: &[112, 0, 80, 0, 64, 0, 80, 0, 112, 0],
        },
        101 => &EChar {
            width: 4,
            height: 5,
            top: 13,
            buf: &[112, 0, 80, 0, 112, 0, 64, 0, 112, 0],
        },
        102 => &EChar {
            width: 3,
            height: 7,
            top: 11,
            buf: &[32, 0, 64, 0, 224, 0, 64, 0, 64, 0, 64, 0, 64, 0],
        },
        104 => &EChar {
            width: 4,
            height: 7,
            top: 11,
            buf: &[64, 0, 64, 0, 112, 0, 80, 0, 80, 0, 80, 0, 80, 0],
        },
        105 => &EChar {
            width: 2,
            height: 7,
            top: 11,
            buf: &[64, 0, 0, 0, 64, 0, 64, 0, 64, 0, 64, 0, 64, 0],
        },
        107 => &EChar {
            width: 4,
            height: 7,
            top: 11,
            buf: &[64, 0, 64, 0, 80, 0, 80, 0, 96, 0, 80, 0, 80, 0],
        },
        108 => &EChar {
            width: 2,
            height: 7,
            top: 11,
            buf: &[64, 0, 64, 0, 64, 0, 64, 0, 64, 0, 64, 0, 64, 0],
        },
        110 => &EChar {
            width: 4,
            height: 5,
            top: 13,
            buf: &[112, 0, 80, 0, 80, 0, 80, 0, 80, 0],
        },
        111 => &EChar {
            width: 4,
            height: 5,
            top: 13,
            buf: &[112, 0, 80, 0, 80, 0, 80, 0, 112, 0],
        },
        112 => &EChar {
            width: 4,
            height: 7,
            top: 13,
            buf: &[112, 0, 80, 0, 80, 0, 80, 0, 112, 0, 64, 0, 64, 0],
        },
        114 => &EChar {
            width: 3,
            height: 5,
            top: 13,
            buf: &[112, 0, 64, 0, 64, 0, 64, 0, 64, 0],
        },
        115 => &EChar {
            width: 4,
            height: 5,
            top: 13,
            buf: &[112, 0, 64, 0, 112, 0, 16, 0, 112, 0],
        },
        116 => &EChar {
            width: 3,
            height: 6,
            top: 12,
            buf: &[64, 0, 224, 0, 64, 0, 64, 0, 64, 0, 32, 0],
        },
        117 => &EChar {
            width: 4,
            height: 5,
            top: 13,
            buf: &[80, 0, 80, 0, 80, 0, 80, 0, 112, 0],
        },
        _ => &ECHAR_NULL,
    }
}

pub fn print_eset(needed: &[u8], set: &ESet, name: &str) {
    println!("match {} {{", name);
    for cval in needed.iter().copied() {
        let chr = &set.chars[cval as usize];
        print!("  {} => &EChar{{", cval);
        print!(" width: {},", chr.width);
        print!(" height: {},", chr.height);
        print!(" top: {},", chr.top);
        println!(" buf: &{:?}}},", chr.buf);
    }
    println!("  _ => panic!(),");
    println!("}}");
}

pub struct Draw {
    commands: &'static [(u16, u16, u8, KbCset)],
    width: u32,
    height: u32,
    xoff: u16,
}

pub const KB_DRAW: Draw = Draw {
    commands: &KB_DRAW_COMMANDS,
    width: 580,
    height: 175,
    xoff: 0,
};

pub const NP_DRAW: Draw = Draw {
    commands: &NP_DRAW_COMMANDS,
    width: 130,
    height: 175,
    xoff: 599,
};

impl Draw {
    pub fn to_page(&self, eset: &ESet) -> Result<Page, DrawPrintErr> {
        let mut page = Page::new(self.width, self.height);
        for (x, y, cval, cset) in self.commands.iter().copied() {
            let echr = match cset {
                KbCset::Selected => &eset.chars[cval as usize],
                KbCset::Graphics => get_gchar(cval),
                KbCset::Labels => get_lchar(cval),
            };
            let x = x + 6 - self.xoff;
            let y = y - 52;
            if let Err(e) = page.draw_echar(x, y, echr) {
                let (w, h) = (self.width, self.height);
                log::warn!("Failed to draw {cval} at ({x},{y}) of ({w}x{h}): {e}");
            }
        }
        Ok(page)
    }
}
