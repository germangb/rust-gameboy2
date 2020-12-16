const UNDEF: u16 = 0x00;

macro_rules! branch {
    ($l:expr, $r:expr) => {
        ($l << 8) | $r
    };
}

#[rustfmt::skip]
const PREFIXED: &[u16; 256] = &[
    //       x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, xA, xB, xC, xD, xE, xF
    /* 0x */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* 1x */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* 2x */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* 3x */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* 4x */ 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_,
    /* 5x */ 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_,
    /* 6x */ 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_,
    /* 7x */ 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_,
    /* 8x */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* 9x */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* Ax */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* Bx */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* Cx */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* Dx */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* Ex */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* Fx */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
];

#[rustfmt::skip]
const UNPREFIXED: &[u16; 256] = &[
    //       x0,             x1, x2,              x3,    x4,              x5, x6, x7, x8,             x9, xA,              xB,    xC,              xD,    xE, xF
    /* 0x */ 4_,             12, 8_,              8_,    4_,              4_, 8_, 4_, 20,             8_, 8_,              8_,    4_,              4_,    8_, 4_,
    /* 1x */ 4_,             12, 8_,              8_,    4_,              4_, 8_, 4_, 12,             8_, 8_,              8_,    4_,              4_,    8_, 4_,
    /* 2x */ branch!(12, 8), 12, 8_,              8_,    4_,              4_, 8_, 4_, branch!(12, 8), 8_, 8_,              8_,    4_,              4_,    8_, 4_,
    /* 3x */ branch!(12, 8), 12, 8_,              8_,    12,              12, 12, 4_, branch!(12, 8), 8_, 8_,              8_,    4_,              4_,    8_, 4_,
    /* 4x */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* 5x */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* 6x */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* 7x */ 8_,             8_, 8_,              8_,    8_,              8_, 4_, 8_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* 8x */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* 9x */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* Ax */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* Bx */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* Cx */ branch!(20, 8), 12, branch!(16, 12), 16,    branch!(24, 12), 16, 8_, 16, branch!(20, 8), 16, branch!(16, 12), 4_,    branch!(24, 12), 24,    8_, 16,
    /* Dx */ branch!(20, 8), 12, branch!(16, 12), UNDEF, branch!(24, 12), 16, 8_, 16, branch!(20, 8), 16, branch!(16, 12), UNDEF, branch!(24, 12), UNDEF, 8_, 16,
    /* Ex */ 12,             12, 8_,              UNDEF, UNDEF,           16, 8_, 16, 16,             4_, 16,              UNDEF, UNDEF,           UNDEF, 8_, 16,
    /* Fx */ 12,             12, 8_,              4_,    4_,              16, 8_, 16, 12,             8_, 16,              4_,    UNDEF,           UNDEF, 8_, 16,
];

pub fn prefixed(opcode: u8) -> u64 {
    let cycles = PREFIXED[opcode as usize];
    assert!(cycles != UNDEF);
    cycles as _
}

pub fn unprefixed(opcode: u8, branch: bool) -> u64 {
    let mut cycles = UNPREFIXED[opcode as usize];
    if branch {
        cycles >>= 8;
    }
    let cycles = cycles & 0xff;
    assert!(cycles != UNDEF);
    cycles as _
}

#[cfg(test)]
mod test {
    use super::{prefixed, unprefixed};

    #[test]
    fn timing_prefixed() {
        // generated using the commands:
        // > curl curl -sS https://raw.githubusercontent.com/lmmendes/game-boy-opcodes/master/opcodes.json | jq ".cbprefixed[].cycles[0]" | grep -n "" | awk -F: '{ print "assert_eq!("$2", prefixed("$1-1"));" }'
        // some of the cycle counts do not match the ones in the test roms.
        // in those cases, the cycle count from the test roms are considered to be
        // correct.

        assert_eq!(8, prefixed(0));
        assert_eq!(8, prefixed(1));
        assert_eq!(8, prefixed(2));
        assert_eq!(8, prefixed(3));
        assert_eq!(8, prefixed(4));
        assert_eq!(8, prefixed(5));
        assert_eq!(16, prefixed(6));
        assert_eq!(8, prefixed(7));
        assert_eq!(8, prefixed(8));
        assert_eq!(8, prefixed(9));
        assert_eq!(8, prefixed(10));
        assert_eq!(8, prefixed(11));
        assert_eq!(8, prefixed(12));
        assert_eq!(8, prefixed(13));
        assert_eq!(16, prefixed(14));
        assert_eq!(8, prefixed(15));
        assert_eq!(8, prefixed(16));
        assert_eq!(8, prefixed(17));
        assert_eq!(8, prefixed(18));
        assert_eq!(8, prefixed(19));
        assert_eq!(8, prefixed(20));
        assert_eq!(8, prefixed(21));
        assert_eq!(16, prefixed(22));
        assert_eq!(8, prefixed(23));
        assert_eq!(8, prefixed(24));
        assert_eq!(8, prefixed(25));
        assert_eq!(8, prefixed(26));
        assert_eq!(8, prefixed(27));
        assert_eq!(8, prefixed(28));
        assert_eq!(8, prefixed(29));
        assert_eq!(16, prefixed(30));
        assert_eq!(8, prefixed(31));
        assert_eq!(8, prefixed(32));
        assert_eq!(8, prefixed(33));
        assert_eq!(8, prefixed(34));
        assert_eq!(8, prefixed(35));
        assert_eq!(8, prefixed(36));
        assert_eq!(8, prefixed(37));
        assert_eq!(16, prefixed(38));
        assert_eq!(8, prefixed(39));
        assert_eq!(8, prefixed(40));
        assert_eq!(8, prefixed(41));
        assert_eq!(8, prefixed(42));
        assert_eq!(8, prefixed(43));
        assert_eq!(8, prefixed(44));
        assert_eq!(8, prefixed(45));
        assert_eq!(16, prefixed(46));
        assert_eq!(8, prefixed(47));
        assert_eq!(8, prefixed(48));
        assert_eq!(8, prefixed(49));
        assert_eq!(8, prefixed(50));
        assert_eq!(8, prefixed(51));
        assert_eq!(8, prefixed(52));
        assert_eq!(8, prefixed(53));
        assert_eq!(16, prefixed(54));
        assert_eq!(8, prefixed(55));
        assert_eq!(8, prefixed(56));
        assert_eq!(8, prefixed(57));
        assert_eq!(8, prefixed(58));
        assert_eq!(8, prefixed(59));
        assert_eq!(8, prefixed(60));
        assert_eq!(8, prefixed(61));
        assert_eq!(16, prefixed(62));
        assert_eq!(8, prefixed(63));
        assert_eq!(8, prefixed(64));
        assert_eq!(8, prefixed(65));
        assert_eq!(8, prefixed(66));
        assert_eq!(8, prefixed(67));
        assert_eq!(8, prefixed(68));
        assert_eq!(8, prefixed(69));
        assert_eq!(12, prefixed(70));
        assert_eq!(8, prefixed(71));
        assert_eq!(8, prefixed(72));
        assert_eq!(8, prefixed(73));
        assert_eq!(8, prefixed(74));
        assert_eq!(8, prefixed(75));
        assert_eq!(8, prefixed(76));
        assert_eq!(8, prefixed(77));
        assert_eq!(12, prefixed(78));
        assert_eq!(8, prefixed(79));
        assert_eq!(8, prefixed(80));
        assert_eq!(8, prefixed(81));
        assert_eq!(8, prefixed(82));
        assert_eq!(8, prefixed(83));
        assert_eq!(8, prefixed(84));
        assert_eq!(8, prefixed(85));
        assert_eq!(12, prefixed(86));
        assert_eq!(8, prefixed(87));
        assert_eq!(8, prefixed(88));
        assert_eq!(8, prefixed(89));
        assert_eq!(8, prefixed(90));
        assert_eq!(8, prefixed(91));
        assert_eq!(8, prefixed(92));
        assert_eq!(8, prefixed(93));
        assert_eq!(12, prefixed(94));
        assert_eq!(8, prefixed(95));
        assert_eq!(8, prefixed(96));
        assert_eq!(8, prefixed(97));
        assert_eq!(8, prefixed(98));
        assert_eq!(8, prefixed(99));
        assert_eq!(8, prefixed(100));
        assert_eq!(8, prefixed(101));
        assert_eq!(12, prefixed(102));
        assert_eq!(8, prefixed(103));
        assert_eq!(8, prefixed(104));
        assert_eq!(8, prefixed(105));
        assert_eq!(8, prefixed(106));
        assert_eq!(8, prefixed(107));
        assert_eq!(8, prefixed(108));
        assert_eq!(8, prefixed(109));
        assert_eq!(12, prefixed(110));
        assert_eq!(8, prefixed(111));
        assert_eq!(8, prefixed(112));
        assert_eq!(8, prefixed(113));
        assert_eq!(8, prefixed(114));
        assert_eq!(8, prefixed(115));
        assert_eq!(8, prefixed(116));
        assert_eq!(8, prefixed(117));
        assert_eq!(12, prefixed(118));
        assert_eq!(8, prefixed(119));
        assert_eq!(8, prefixed(120));
        assert_eq!(8, prefixed(121));
        assert_eq!(8, prefixed(122));
        assert_eq!(8, prefixed(123));
        assert_eq!(8, prefixed(124));
        assert_eq!(8, prefixed(125));
        assert_eq!(12, prefixed(126));
        assert_eq!(8, prefixed(127));
        assert_eq!(8, prefixed(128));
        assert_eq!(8, prefixed(129));
        assert_eq!(8, prefixed(130));
        assert_eq!(8, prefixed(131));
        assert_eq!(8, prefixed(132));
        assert_eq!(8, prefixed(133));
        assert_eq!(16, prefixed(134));
        assert_eq!(8, prefixed(135));
        assert_eq!(8, prefixed(136));
        assert_eq!(8, prefixed(137));
        assert_eq!(8, prefixed(138));
        assert_eq!(8, prefixed(139));
        assert_eq!(8, prefixed(140));
        assert_eq!(8, prefixed(141));
        assert_eq!(16, prefixed(142));
        assert_eq!(8, prefixed(143));
        assert_eq!(8, prefixed(144));
        assert_eq!(8, prefixed(145));
        assert_eq!(8, prefixed(146));
        assert_eq!(8, prefixed(147));
        assert_eq!(8, prefixed(148));
        assert_eq!(8, prefixed(149));
        assert_eq!(16, prefixed(150));
        assert_eq!(8, prefixed(151));
        assert_eq!(8, prefixed(152));
        assert_eq!(8, prefixed(153));
        assert_eq!(8, prefixed(154));
        assert_eq!(8, prefixed(155));
        assert_eq!(8, prefixed(156));
        assert_eq!(8, prefixed(157));
        assert_eq!(16, prefixed(158));
        assert_eq!(8, prefixed(159));
        assert_eq!(8, prefixed(160));
        assert_eq!(8, prefixed(161));
        assert_eq!(8, prefixed(162));
        assert_eq!(8, prefixed(163));
        assert_eq!(8, prefixed(164));
        assert_eq!(8, prefixed(165));
        assert_eq!(16, prefixed(166));
        assert_eq!(8, prefixed(167));
        assert_eq!(8, prefixed(168));
        assert_eq!(8, prefixed(169));
        assert_eq!(8, prefixed(170));
        assert_eq!(8, prefixed(171));
        assert_eq!(8, prefixed(172));
        assert_eq!(8, prefixed(173));
        assert_eq!(16, prefixed(174));
        assert_eq!(8, prefixed(175));
        assert_eq!(8, prefixed(176));
        assert_eq!(8, prefixed(177));
        assert_eq!(8, prefixed(178));
        assert_eq!(8, prefixed(179));
        assert_eq!(8, prefixed(180));
        assert_eq!(8, prefixed(181));
        assert_eq!(16, prefixed(182));
        assert_eq!(8, prefixed(183));
        assert_eq!(8, prefixed(184));
        assert_eq!(8, prefixed(185));
        assert_eq!(8, prefixed(186));
        assert_eq!(8, prefixed(187));
        assert_eq!(8, prefixed(188));
        assert_eq!(8, prefixed(189));
        assert_eq!(16, prefixed(190));
        assert_eq!(8, prefixed(191));
        assert_eq!(8, prefixed(192));
        assert_eq!(8, prefixed(193));
        assert_eq!(8, prefixed(194));
        assert_eq!(8, prefixed(195));
        assert_eq!(8, prefixed(196));
        assert_eq!(8, prefixed(197));
        assert_eq!(16, prefixed(198));
        assert_eq!(8, prefixed(199));
        assert_eq!(8, prefixed(200));
        assert_eq!(8, prefixed(201));
        assert_eq!(8, prefixed(202));
        assert_eq!(8, prefixed(203));
        assert_eq!(8, prefixed(204));
        assert_eq!(8, prefixed(205));
        assert_eq!(16, prefixed(206));
        assert_eq!(8, prefixed(207));
        assert_eq!(8, prefixed(208));
        assert_eq!(8, prefixed(209));
        assert_eq!(8, prefixed(210));
        assert_eq!(8, prefixed(211));
        assert_eq!(8, prefixed(212));
        assert_eq!(8, prefixed(213));
        assert_eq!(16, prefixed(214));
        assert_eq!(8, prefixed(215));
        assert_eq!(8, prefixed(216));
        assert_eq!(8, prefixed(217));
        assert_eq!(8, prefixed(218));
        assert_eq!(8, prefixed(219));
        assert_eq!(8, prefixed(220));
        assert_eq!(8, prefixed(221));
        assert_eq!(16, prefixed(222));
        assert_eq!(8, prefixed(223));
        assert_eq!(8, prefixed(224));
        assert_eq!(8, prefixed(225));
        assert_eq!(8, prefixed(226));
        assert_eq!(8, prefixed(227));
        assert_eq!(8, prefixed(228));
        assert_eq!(8, prefixed(229));
        assert_eq!(16, prefixed(230));
        assert_eq!(8, prefixed(231));
        assert_eq!(8, prefixed(232));
        assert_eq!(8, prefixed(233));
        assert_eq!(8, prefixed(234));
        assert_eq!(8, prefixed(235));
        assert_eq!(8, prefixed(236));
        assert_eq!(8, prefixed(237));
        assert_eq!(16, prefixed(238));
        assert_eq!(8, prefixed(239));
        assert_eq!(8, prefixed(240));
        assert_eq!(8, prefixed(241));
        assert_eq!(8, prefixed(242));
        assert_eq!(8, prefixed(243));
        assert_eq!(8, prefixed(244));
        assert_eq!(8, prefixed(245));
        assert_eq!(16, prefixed(246));
        assert_eq!(8, prefixed(247));
        assert_eq!(8, prefixed(248));
        assert_eq!(8, prefixed(249));
        assert_eq!(8, prefixed(250));
        assert_eq!(8, prefixed(251));
        assert_eq!(8, prefixed(252));
        assert_eq!(8, prefixed(253));
        assert_eq!(16, prefixed(254));
        assert_eq!(8, prefixed(255));
    }

    #[test]
    fn timing_unprefixed() {
        todo!()
    }
}
