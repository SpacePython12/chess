use std::{collections::HashMap, sync::{LazyLock, OnceLock, RwLock}};

use crate::chess::{bitboards::BitBoard, position::*};
use super::*;

const KNIGHT_JUMPS: [Offset; 8] = [
    Offset::new(-2, -1),
    Offset::new(-2,  1),
    Offset::new(-1,  2),
    Offset::new( 1,  2),
    Offset::new( 2,  1),
    Offset::new( 2, -1),
    Offset::new( 1, -2),
    Offset::new(-1, -2),
];

// const ORTHOGONAL_SHIFTS: [u32; 64] = [52, 52, 52, 52, 52, 52, 52, 52, 53, 53, 53, 54, 53, 53, 54, 53, 53, 54, 54, 54, 53, 53, 54, 53, 53, 54, 53, 53, 54, 54, 54, 53, 52, 54, 53, 53, 53, 53, 54, 53, 52, 53, 54, 54, 53, 53, 54, 53, 53, 54, 54, 54, 53, 53, 54, 53, 52, 53, 53, 53, 53, 53, 53, 52];
// const DIAGONAL_SHIFTS: [u32; 64] = [58, 60, 59, 59, 59, 59, 60, 58, 60, 59, 59, 59, 59, 59, 59, 60, 59, 59, 57, 57, 57, 57, 59, 59, 59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 57, 57, 57, 59, 59, 60, 60, 59, 59, 59, 59, 60, 60, 58, 60, 59, 59, 59, 59, 59, 58];

// const ORTHOGONAL_MAGICS: [u64; 64] = [0x68000814008b4a0, 0xffbfffdff7efdffd, 0x232000ac16a02028, 0x5490000442001110, 0xbfeffe7bfcf5f9f7, 0xdfe7dff7faffff7b, 0xafefeff77fbdfff9, 0x4b00017282420100, 0x780084002a984, 0xffbff7dfbf6fe3be, 0xefffaffeef9ffdff, 0x448a002040b2002a, 0x6ad97fd7fbff7fde, 0x7dfdfffdfef7efff, 0x806000200010c68, 0x83ca800080014300, 0x1008208000400199, 0x8002808020044004, 0x197a020040322180, 0x710c2000a001122, 0xe6d7fdffdffbf5ff, 0xffffbfeefdfc7fff, 0x858e34006a100f08, 0x4083a20002841043, 0xa4000c480008365, 0x180ca08100400508, 0x2150717010040800, 0x48001000a0040a18, 0x908801190011001c, 0x10220082005c0810, 0x8268502400281302, 0x8a341820010490c, 0x6effdff7afbfff3e, 0x6b21a010084000c3, 0xe090000880c5e000, 0xffeeffaffcfffbdf, 0x4807830010040, 0xffffeffbf7ff5fbf, 0x10d91210d4000308, 0x4088938502000444, 0x860184a040544000, 0xfff7bff77df677ff, 0x33486280b2020041, 0x9310210470010018, 0xffd9fefb23b1fffe, 0xffffeffdfffb7fdf, 0x3000c21d08040010, 0x8b00058c410016, 0x800e8002a9400680, 0x59e2c4ae01008200, 0x11a110a0c2048200, 0x280083b000680080, 0xa04008a014265001, 0x19c4088200010040, 0x925082a303c00, 0x448221840ac90600, 0xbdfffefbbddd7fff, 0xdbffff3fefd7bfff, 0xfdffffdfffbfafb5, 0xaffffeff9fefff57, 0xbfeffffdf7fa9d6e, 0xfdffffe7fbfffeff, 0x6df7fff7fefdeffc, 0x41b3988401214702]; 
// const DIAGONAL_MAGICS: [u64; 64] = [0xe51ebb94fbe45bff, 0xc7b9f567ed8ffe7f, 0x19a8282157800224, 0x4d41401923c73be, 0x480404a14244000d, 0x340e01fea0c933af, 0xffc3f989d57fe9ec, 0xf7ff3fdd6efbffff, 0xe7f974f4f9d9f7f5, 0x222161180311c580, 0x62003808704081a0, 0x70425c0408830360, 0x66e5e110419250cd, 0x8420220834154952, 0xede7f5adf8fdfffd, 0x7f7eea5f3d59bf5e, 0x4478024050810631, 0x2a22030490025601, 0xd0e4044848002500, 0x518c08c801212289, 0x1002004402111108, 0x5858102901009007, 0x2230a1c412051005, 0x142b042e41082700, 0x882011ae8850a508, 0x274320c508181108, 0x80090048e040014, 0x508c00c01c0100b2, 0x424300102b004000, 0x490144022080230, 0x20c280b11c020814, 0x20600c42c9212, 0xf3bfc5c66b10122f, 0xa25801b000c42434, 0x500144a208900400, 0x6080140400780120, 0x124c0b40100c0100, 0x7048004100909018, 0xe8781ec402008a01, 0x4d221a0600807284, 0xed4ff5c4ea9b2418, 0x6d9fe6f7b7efdeb4, 0x433d610048044041, 0x86a2018040300, 0x214b14110c006200, 0x60d2241106001c0a, 0x2b7f1018f2ebfdcc, 0x783438008e252100, 0xefbffd71eead7fff, 0xd597fe7d435f79ff, 0x9ffff7fb3d9c7377, 0x8910144620981000, 0xe40d01a0208a40f9, 0xa63240b31425025c, 0x9effebf61dd769fb, 0x877fce36d752fa8e, 0xeb5ffd776d5fdfbe, 0xd7fdfffebcf2deff, 0xd01801c04208900f, 0x830639ef5720980a, 0xc9c018202182c400, 0x893b2d4094880e8c, 0xfbbefdf552eb5ae6, 0xfefffbfb7bdfddfb];

// const ORTHOGONAL_MAGICS: [u64; 64] = [
//     0x00280077ffebfffe,
//     0x2004010201097fff,
//     0x0010020010053fff,
//     0x0040040008004002,
//     0x7fd00441ffffd003,
//     0x4020008887dffffe,
//     0x004000888847ffff,
//     0x006800fbff75fffd,
//     0x000028010113ffff,
//     0x0020040201fcffff,
//     0x007fe80042ffffe8,
//     0x00001800217fffe8,
//     0x00001800073fffe8,
//     0x00001800e05fffe8,
//     0x00001800602fffe8,
//     0x000030002fffffa0,
//     0x00300018010bffff,
//     0x0003000c0085fffb,
//     0x0004000802010008,
//     0x0004002020020004,
//     0x0001002002002001,
//     0x0001001000801040,
//     0x0000004040008001,
//     0x0000006800cdfff4,
//     0x0040200010080010,
//     0x0000080010040010,
//     0x0004010008020008,
//     0x0000040020200200,
//     0x0002008010100100,
//     0x0000008020010020,
//     0x0000008020200040,
//     0x0000820020004020,
//     0x00fffd1800300030,
//     0x007fff7fbfd40020,
//     0x003fffbd00180018,
//     0x001fffde80180018,
//     0x000fffe0bfe80018,
//     0x0001000080202001,
//     0x0003fffbff980180,
//     0x0001fffdff9000e0,
//     0x00fffefeebffd800,
//     0x007ffff7ffc01400,
//     0x003fffbfe4ffe800,
//     0x001ffff01fc03000,
//     0x000fffe7f8bfe800,
//     0x0007ffdfdf3ff808,
//     0x0003fff85fffa804,
//     0x0001fffd75ffa802,
//     0x00ffffd7ffebffd8,
//     0x007fff75ff7fbfd8,
//     0x003fff863fbf7fd8,
//     0x001fffbfdfd7ffd8,
//     0x000ffff810280028,
//     0x0007ffd7f7feffd8,
//     0x0003fffc0c480048,
//     0x0001ffffafd7ffd8,
//     0x00ffffe4ffdfa3ba,
//     0x007fffef7ff3d3da,
//     0x003fffbfdfeff7fa,
//     0x001fffeff7fbfc22,
//     0x0000020408001001,
//     0x0007fffeffff77fd,
//     0x0003ffffbf7dfeec,
//     0x0001ffff9dffa333,
// ];
// const DIAGONAL_MAGICS: [u64; 64] = [
//     0x007fbfbfbfbfbfff,
//     0x0000a060401007fc,
//     0x0001004008020000,
//     0x0000806004000000,
//     0x0000100400000000,
//     0x000021c100b20000,
//     0x0000040041008000,
//     0x00000fb0203fff80,
//     0x0000040100401004,
//     0x0000020080200802,
//     0x0000004010202000,
//     0x0000008060040000,
//     0x0000004402000000,
//     0x0000000801008000,
//     0x000007efe0bfff80,
//     0x0000000820820020,
//     0x0000400080808080,
//     0x00021f0100400808,
//     0x00018000c06f3fff,
//     0x0000258200801000,
//     0x0000240080840000,
//     0x000018000c03fff8,
//     0x00000a5840208020,
//     0x0000020008208020,
//     0x0000804000810100,
//     0x0001011900802008,
//     0x0000804000810100,
//     0x000100403c0403ff,
//     0x00078402a8802000,
//     0x0000101000804400,
//     0x0000080800104100,
//     0x00004004c0082008,
//     0x0001010120008020,
//     0x000080809a004010,
//     0x0007fefe08810010,
//     0x0003ff0f833fc080,
//     0x007fe08019003042,
//     0x003fffefea003000,
//     0x0000101010002080,
//     0x0000802005080804,
//     0x0000808080a80040,
//     0x0000104100200040,
//     0x0003ffdf7f833fc0,
//     0x0000008840450020,
//     0x00007ffc80180030,
//     0x007fffdd80140028,
//     0x00020080200a0004,
//     0x0000101010100020,
//     0x0007ffdfc1805000,
//     0x0003ffefe0c02200,
//     0x0000000820806000,
//     0x0000000008403000,
//     0x0000000100202000,
//     0x0000004040802000,
//     0x0004010040100400,
//     0x00006020601803f4,
//     0x0003ffdfdfc28048,
//     0x0000000820820020,
//     0x0000000008208060,
//     0x0000000000808020,
//     0x0000000001002020,
//     0x0000000401002008,
//     0x0000004040404040,
//     0x007fff9fdf7ff813,
// ];

struct MagicEntry {
    neg_mask: u64,
    magic: u64,
    offset: u32
}

const EMPTY_MAGIC_ENTRY: MagicEntry = MagicEntry {
    neg_mask: 0,
    magic: 0,
    offset: 0
};

// const fns can't be parameterized by functions, so we use a macro instead.
macro_rules! gen_entries {
    ($relevant_blockers:ident, $raw_magics:expr) => {{
        let raw_magics = $raw_magics;
        let mut magics = [EMPTY_MAGIC_ENTRY; Position::NUM];
        let mut i = 0;
        while i < raw_magics.len() {
            let neg_mask = !$relevant_blockers[i];
            let (magic, offset) = raw_magics[i];
            magics[i] = MagicEntry { neg_mask, magic, offset };
            i += 1;
        }
        magics
    }};
}

// Black magics found by Volker Annuss and Niklas Fiekas
// http://talkchess.com/forum/viewtopic.php?t=64790

const ORTHOGONAL_MAGICS: &[MagicEntry; Position::NUM] = &gen_entries!(
    ORTHOGONAL_MASKS,
    [
        (0x80280013FF84FFFF, 10890), (0x5FFBFEFDFEF67FFF, 50579), (0xFFEFFAFFEFFDFFFF, 62020),
        (0x003000900300008A, 67322), (0x0050028010500023, 80251), (0x0020012120A00020, 58503),
        (0x0030006000C00030, 51175), (0x0058005806B00002, 83130), (0x7FBFF7FBFBEAFFFC, 50430),
        (0x0000140081050002, 21613), (0x0000180043800048, 72625), (0x7FFFE800021FFFB8, 80755),
        (0xFFFFCFFE7FCFFFAF, 69753), (0x00001800C0180060, 26973), (0x4F8018005FD00018, 84972),
        (0x0000180030620018, 31958), (0x00300018010C0003, 69272), (0x0003000C0085FFFF, 48372),
        (0xFFFDFFF7FBFEFFF7, 65477), (0x7FC1FFDFFC001FFF, 43972), (0xFFFEFFDFFDFFDFFF, 57154),
        (0x7C108007BEFFF81F, 53521), (0x20408007BFE00810, 30534), (0x0400800558604100, 16548),
        (0x0040200010080008, 46407), (0x0010020008040004, 11841), (0xFFFDFEFFF7FBFFF7, 21112),
        (0xFEBF7DFFF8FEFFF9, 44214), (0xC00000FFE001FFE0, 57925), (0x4AF01F00078007C3, 29574),
        (0xBFFBFAFFFB683F7F, 17309), (0x0807F67FFA102040, 40143), (0x200008E800300030, 64659),
        (0x0000008780180018, 70469), (0x0000010300180018, 62917), (0x4000008180180018, 60997),
        (0x008080310005FFFA, 18554), (0x4000188100060006, 14385), (0xFFFFFF7FFFBFBFFF,     0),
        (0x0000802000200040, 38091), (0x20000202EC002800, 25122), (0xFFFFF9FF7CFFF3FF, 60083),
        (0x000000404B801800, 72209), (0x2000002FE03FD000, 67875), (0xFFFFFF6FFE7FCFFD, 56290),
        (0xBFF7EFFFBFC00FFF, 43807), (0x000000100800A804, 73365), (0x6054000A58005805, 76398),
        (0x0829000101150028, 20024), (0x00000085008A0014,  9513), (0x8000002B00408028, 24324),
        (0x4000002040790028, 22996), (0x7800002010288028, 23213), (0x0000001800E08018, 56002),
        (0xA3A80003F3A40048, 22809), (0x2003D80000500028, 44545), (0xFFFFF37EEFEFDFBE, 36072),
        (0x40000280090013C1,  4750), (0xBF7FFEFFBFFAF71F,  6014), (0xFFFDFFFF777B7D6E, 36054),
        (0x48300007E8080C02, 78538), (0xAFE0000FFF780402, 28745), (0xEE73FFFBFFBB77FE,  8555),
        (0x0002000308482882,  1009)
    ]
);

const DIAGONAL_MAGICS: &[MagicEntry; Position::NUM] = &gen_entries!(
    DIAGONAL_MASKS,
    [
        (0xA7020080601803D8, 60984), (0x13802040400801F1, 66046), (0x0A0080181001F60C, 32910),
        (0x1840802004238008, 16369), (0xC03FE00100000000, 42115), (0x24C00BFFFF400000,   835),
        (0x0808101F40007F04, 18910), (0x100808201EC00080, 25911), (0xFFA2FEFFBFEFB7FF, 63301),
        (0x083E3EE040080801, 16063), (0xC0800080181001F8, 17481), (0x0440007FE0031000, 59361),
        (0x2010007FFC000000, 18735), (0x1079FFE000FF8000, 61249), (0x3C0708101F400080, 68938),
        (0x080614080FA00040, 61791), (0x7FFE7FFF817FCFF9, 21893), (0x7FFEBFFFA01027FD, 62068),
        (0x53018080C00F4001, 19829), (0x407E0001000FFB8A, 26091), (0x201FE000FFF80010, 15815),
        (0xFFDFEFFFDE39FFEF, 16419), (0xCC8808000FBF8002, 59777), (0x7FF7FBFFF8203FFF, 16288),
        (0x8800013E8300C030, 33235), (0x0420009701806018, 15459), (0x7FFEFF7F7F01F7FD, 15863),
        (0x8700303010C0C006, 75555), (0xC800181810606000, 79445), (0x20002038001C8010, 15917),
        (0x087FF038000FC001,  8512), (0x00080C0C00083007, 73069), (0x00000080FC82C040, 16078),
        (0x000000407E416020, 19168), (0x00600203F8008020, 11056), (0xD003FEFE04404080, 62544),
        (0xA00020C018003088, 80477), (0x7FBFFE700BFFE800, 75049), (0x107FF00FE4000F90, 32947),
        (0x7F8FFFCFF1D007F8, 59172), (0x0000004100F88080, 55845), (0x00000020807C4040, 61806),
        (0x00000041018700C0, 73601), (0x0010000080FC4080, 15546), (0x1000003C80180030, 45243),
        (0xC10000DF80280050, 20333), (0xFFFFFFBFEFF80FDC, 33402), (0x000000101003F812, 25917),
        (0x0800001F40808200, 32875), (0x084000101F3FD208,  4639), (0x080000000F808081, 17077),
        (0x0004000008003F80, 62324), (0x08000001001FE040, 18159), (0x72DD000040900A00, 61436),
        (0xFFFFFEFFBFEFF81D, 57073), (0xCD8000200FEBF209, 61025), (0x100000101EC10082, 81259),
        (0x7FBAFFFFEFE0C02F, 64083), (0x7F83FFFFFFF07F7F, 56114), (0xFFF1FFFFFFF7FFC1, 57058),
        (0x0878040000FFE01F, 58912), (0x945E388000801012, 22194), (0x0840800080200FDA, 70880),
        (0x100000C05F582008, 11140)
    ]
);

const ORTHOGONAL_INDEX_BITS: usize = 12;

const DIAGONAL_INDEX_BITS: usize = 9;

const fn get_magic_index(coord_index: usize, blockers: u64, ortho: bool) -> usize {
    let (magics, index_bits) = if ortho {
        (ORTHOGONAL_MAGICS, ORTHOGONAL_INDEX_BITS)
    } else {
        (DIAGONAL_MAGICS, DIAGONAL_INDEX_BITS)
    };
    let magic = &magics[coord_index];
    let relevant_blockers = blockers | magic.neg_mask;
    let hash = relevant_blockers.wrapping_mul(magic.magic);
    magic.offset as usize + (hash >> (Position::NUM - index_bits)) as usize
}

const fn create_movement_mask(coord_index: usize, ortho: bool) -> u64 {
    let mut mask = BitBoard::new();
    let directions = if ortho {
        Offset::ORTHO_DIRECTIONS
    } else {
        Offset::DIAG_DIRECTIONS
    };
    let src = Position::from_index(coord_index as u8);
    let mut i = 0usize;
    while i < 4 {
        {
            let dir = directions[i];
            let mut dist = 1i8;
            while dist < 8 {
                if let Some(dst) = src.checked_offset(dir.with_magnitude(dist)) {
                    mask.set(dst);
                    dist += 1;
                } else {
                    break;
                }
            }
        }
        i += 1;
    }
    mask.0
}

const fn create_movement_masks(ortho: bool) -> [u64; 64] {
    let mut masks = [0; 64];
    let mut i = 0usize;
    while i < 64 {
        masks[i] = create_movement_mask(i, ortho);
        i += 1;
    }
    masks
}

const ORTHOGONAL_MASKS: [u64; 64] = create_movement_masks(true);
const DIAGONAL_MASKS: [u64; 64] = create_movement_masks(false);

const KING_MOVES: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut i = 0usize;
    while i < 64 {
        {
            let mut bitboard = BitBoard::new();
            let src = Position::from_index(i as u8);
            
            let mut j = 0usize;
            while j < 4 {
                if let Some(dst) = src.checked_offset(Offset::ORTHO_DIRECTIONS[j]) {
                    bitboard.set(dst);
                }
                if let Some(dst) = src.checked_offset(Offset::DIAG_DIRECTIONS[j]) {
                    bitboard.set(dst);
                }
                j += 1;
            }
            table[i] = bitboard.0;
        }
        i += 1;
    }
    table
};

const KNIGHT_ATTACKS: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut i = 0usize;
    while i < 64 {
        {
            let mut bitboard = BitBoard::new();
            let src = Position::from_index(i as u8);
            
            let mut j = 0usize;
            while j < 8 {
                if let Some(dst) = src.checked_offset(KNIGHT_JUMPS[j]) {
                    bitboard.set(dst);
                }
                j += 1;
            }
            table[i] = bitboard.0;
        }
        i += 1;
    }
    table
};

const WHITE_PAWN_ATTACKS: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut i = 0usize;
    while i < 64 {
        {
            let mut bitboard = BitBoard::new();
            let src = Position::from_index(i as u8);
            
            if let Some(dst) = src.checked_offset(Offset::NW) {
                bitboard.set(dst);
            }
            if let Some(dst) = src.checked_offset(Offset::NE) {
                bitboard.set(dst);
            }
            table[i] = bitboard.0;
        }
        i += 1;
    }
    table
};

const BLACK_PAWN_ATTACKS: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut i = 0usize;
    while i < 64 {
        {
            let mut bitboard = BitBoard::new();
            let src = Position::from_index(i as u8);
            
            if let Some(dst) = src.checked_offset(Offset::SW) {
                bitboard.set(dst);
            }
            if let Some(dst) = src.checked_offset(Offset::SE) {
                bitboard.set(dst);
            }
            table[i] = bitboard.0;
        }
        i += 1;
    }
    table
};

const DIR_RAY_MASKS: [[u64; 8]; 64] = {
    let mut table = [[0u64; 8]; 64];
    let mut i = 0usize;
    while i < 64 {
        let src = Position::from_index(i as u8);
        let mut j = 0usize;
        while j < 8 {
            let dir = Offset::DIRECTIONS[j];
            let mut bitboard = BitBoard::new();

            let mut dist = 0i8;

            while dist < 8 {
                if let Some(dst) = src.checked_offset(dir.with_magnitude(dist)) {
                    bitboard.set(dst);
                }
                dist += 1;
            }

            table[i][j] = bitboard.0;

            j += 1;
        }
        i += 1;
    }
    table
};

const ALIGN_MASKS: [[u64; 8]; 64] = {
    let mut table = [[0u64; 8]; 64];
    let mut i = 0usize;
    while i < 64 {
        let src = Position::from_index(i as u8);
        let mut j = 0usize;
        while j < 8 {
            let dir = Offset::DIRECTIONS[j];
            let mut bitboard = BitBoard::new();

            let mut dist = -7i8;

            while dist < 8 {
                if let Some(dst) = src.checked_offset(dir.with_magnitude(dist)) {
                    bitboard.set(dst);
                }
                dist += 1;
            }

            table[i][j] = bitboard.0;

            j += 1;
        }
        i += 1;
    }
    table
};

const fn create_sliding_moves(coord_index: usize, blockers: u64, ortho: bool) -> u64 {
    let blockers = BitBoard(blockers);
    let mut bitboard = BitBoard::new();
    let directions = if ortho {
        Offset::ORTHO_DIRECTIONS
    } else {
        Offset::DIAG_DIRECTIONS
    };
    let src = Position::from_index(coord_index as u8);
    let mut i = 0;
    while i < 4 {
        let dir = directions[i];
        let mut dist = 1;
        while dist < 8 {
            if let Some(dst) = src.checked_offset(dir.with_magnitude(dist)) {
                bitboard.set(dst);
                if blockers.get(dst) {
                    break;
                }
            } else {
                break;
            }
            dist += 1;
        }
        i += 1;
    }
    bitboard.0

}

const fn write_attacks(table: &mut [BitBoard], ortho: bool) {
    let mut coord_index = 0usize;
    while coord_index < 64 {
        let mut subset_iter = BitBoard(if ortho {
            ORTHOGONAL_MASKS[coord_index]
        } else {
            DIAGONAL_MASKS[coord_index]
        }).iter_subsets();
        while let Some(subset) = subset_iter.const_next() {
            table[get_magic_index(coord_index, subset.0, ortho)] = BitBoard(create_sliding_moves(coord_index, subset.0, ortho));
        }
        coord_index += 1;
    }
}

const SLIDING_MOVE_TABLE_SIZE: usize = 87988;

static SLIDING_MOVES: LazyLock<RwLock<HashMap<(Position, BitBoard, bool), BitBoard>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

fn get_sliding_attacks_worse(pos: Position, blockers: BitBoard, ortho: bool) -> BitBoard {
    let lock = SLIDING_MOVES.read().unwrap();
    let key = (pos, blockers, ortho);
    if lock.contains_key(&key) {
        *lock.get(&key).unwrap()
    } else {
        let val = BitBoard(create_sliding_moves(pos.into_index() as usize, blockers.0, ortho));
        drop(lock);
        SLIDING_MOVES.write().unwrap().insert(key, val);
        val
    }
}

static SLIDING_MOVE_TABLE: [OnceLock<BitBoard>; SLIDING_MOVE_TABLE_SIZE] = [const { OnceLock::new() }; SLIDING_MOVE_TABLE_SIZE];

fn get_sliding_attacks(pos: Position, blockers: BitBoard, ortho: bool) -> BitBoard {
    let index = get_magic_index(pos.into_index() as usize, blockers.0, ortho);
    *SLIDING_MOVE_TABLE[index].get_or_init(|| {
        BitBoard(create_sliding_moves(pos.into_index() as usize, blockers.0, ortho))
    })
}

// fn create_attack_table(coord_index: usize, ortho: bool) {
//     let num_bits = 64 - shift;
//     let lookup_size = 1usize << num_bits;
//     let mut table = vec![0u64; lookup_size];

//     let movement_mask = create_movement_mask(coord_index, ortho);

//     let blocker_patterns = create_all_blocker_bitboards(movement_mask);

//     for pattern in blocker_patterns {
//         let index = get_magic_index(coord_index, pattern, ortho);
//         let moves = create_legal_move_bitboard_from_blockers(coord_index, pattern, ortho);
//         table[index as usize] = moves;
//     }
//     table
// }

pub fn get_orthogonal_attacks(pos: Position, blockers: BitBoard) -> BitBoard {
    get_sliding_attacks_worse(pos, blockers, true)
}

pub fn get_diagonal_attacks(pos: Position, blockers: BitBoard) -> BitBoard {
    get_sliding_attacks_worse(pos, blockers, false)
}

pub fn get_king_moves(pos: Position) -> BitBoard {
    let coord_index = pos.into_index() as usize;
    BitBoard(KING_MOVES[coord_index])
}

pub fn get_knight_attacks(pos: Position) -> BitBoard {
    let coord_index = pos.into_index() as usize;
    BitBoard(KNIGHT_ATTACKS[coord_index])
}

pub fn get_pawn_attacks(pos: Position, color: PieceColor) -> BitBoard {
    let coord_index = pos.into_index() as usize;
    BitBoard(match color {
        PieceColor::White => WHITE_PAWN_ATTACKS[coord_index],
        PieceColor::Black => BLACK_PAWN_ATTACKS[coord_index],
    })
}

pub fn get_pawn_bitboard_attacks(bitboard: BitBoard, color: PieceColor) -> BitBoard {
    BitBoard(match color {
        PieceColor::White => ((bitboard.0 << 9) & !BitBoard::FILE_A.0) | ((bitboard.0 << 7) & !BitBoard::FILE_H.0),
        PieceColor::Black => ((bitboard.0 >> 7) & !BitBoard::FILE_A.0) | ((bitboard.0 >> 9) & !BitBoard::FILE_H.0),
    })
}

pub fn get_dir_ray_mask(src: Position, dst: Position) -> BitBoard {
    let dir = (dst-src).signum();
    BitBoard(DIR_RAY_MASKS[src.into_index() as usize][match dir {
        Offset::N => 0,
        Offset::S => 1,
        Offset::W => 2,
        Offset::E => 3,
        Offset::NW => 4,
        Offset::SE => 5,
        Offset::NE => 6,
        Offset::SW => 7,
        _ => unreachable!()
    }])
}

pub fn get_align_mask(src: Position, dst: Position) -> BitBoard {
    get_dir_ray_mask(src, dst) | get_dir_ray_mask(dst, src)
}