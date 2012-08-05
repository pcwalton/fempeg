//
// fempeg/fempeg.rs
//
// Patrick Walton <pcwalton@mimiga.net>
//
// Copyright (c) 2012 Mozilla Foundation
//
// Based on kjmp2, Copyright (c) 2006 Matrin J. Fieldler <martin.fiedler@gmx.net>
//
// This software is provided 'as-is', without any express or implied         
// warranty. In no event will the authors be held liable for any damages     
// arising from the use of this software.                                    
//                                                                           
// Permission is granted to anyone to use this software for any purpose,     
// including commercial applications, and to alter it and redistribute it    
// freely, subject to the following restrictions:                            
//   1. The origin of this software must not be misrepresented; you must not 
//      claim that you wrote the original software. If you use this software 
//      in a product, an acknowledgment in the product documentation would   
//      be appreciated but is not required.                                  
//   2. Altered source versions must be plainly marked as such, and must not 
//      be misrepresented as being the original software.                    
//   3. This notice may not be removed or altered from any source            
//      distribution.                                                        
//

use ao;
use std;

import None = option::none;
import Some = option::some;
import Error = result::err;
import OK = result::ok;
import Result = result::result;
import vector = vec;

import float::cos;
import i32::range;
import io::println;
import str::from_slice;
import result::unwrap;
import vector::{mut_view, view};

// Simple typedefs

type String = &str;
type UniqueString = ~str;
type MP2Result<T> = Result<T,String>;

// Miscellaneous functions

fn ignore<T>(_x: T) {}

fn abort(error: String) -> ! {
    fail from_slice(error)
}

// Constants

const SAMPLES_PER_FRAME: uint = 1152;

// Modes
enum Mode {
    Stereo,
    JointStereo,
    DualChannel,
    Mono
}

fn Mode(n: i32) -> Mode {
    match n {
        0 => Stereo,
        1 => JointStereo,
        2 => DualChannel,
        3 => Mono,
        _ => abort("invalid mode")
    }
}

// Sample rate table
const SAMPLE_RATES: [i32]/4 = [ 44100, 48000, 32000, 0 ];

// Bitrate table
const BITRATES: [i32]/14 = [ 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384 ];

// Scale factors (24-bit fixed-point)
const SCF_VALUE: [i32]/64 = [
    0x02000000, 0x01965FEA, 0x01428A30, 0x01000000, 0x00CB2FF5, 0x00A14518, 0x00800000, 0x006597FB,
    0x0050A28C, 0x00400000, 0x0032CBFD, 0x00285146, 0x00200000, 0x001965FF, 0x001428A3, 0x00100000,
    0x000CB2FF, 0x000A1451, 0x00080000, 0x00065980, 0x00050A29, 0x00040000, 0x00032CC0, 0x00028514,
    0x00020000, 0x00019660, 0x0001428A, 0x00010000, 0x0000CB30, 0x0000A145, 0x00008000, 0x00006598,
    0x000050A3, 0x00004000, 0x000032CC, 0x00002851, 0x00002000, 0x00001966, 0x00001429, 0x00001000,
    0x00000CB3, 0x00000A14, 0x00000800, 0x00000659, 0x0000050A, 0x00000400, 0x0000032D, 0x00000285,
    0x00000200, 0x00000196, 0x00000143, 0x00000100, 0x000000CB, 0x000000A1, 0x00000080, 0x00000066,
    0x00000051, 0x00000040, 0x00000033, 0x00000028, 0x00000020, 0x00000019, 0x00000014, 0
];

// Synthesis window
const D: [i32]/512 = [
     0x00000, 0x00000, 0x00000, 0x00000, 0x00000, 0x00000, 0x00000,-0x00001,
    -0x00001,-0x00001,-0x00001,-0x00002,-0x00002,-0x00003,-0x00003,-0x00004,
    -0x00004,-0x00005,-0x00006,-0x00006,-0x00007,-0x00008,-0x00009,-0x0000A,
    -0x0000C,-0x0000D,-0x0000F,-0x00010,-0x00012,-0x00014,-0x00017,-0x00019,
    -0x0001C,-0x0001E,-0x00022,-0x00025,-0x00028,-0x0002C,-0x00030,-0x00034,
    -0x00039,-0x0003E,-0x00043,-0x00048,-0x0004E,-0x00054,-0x0005A,-0x00060,
    -0x00067,-0x0006E,-0x00074,-0x0007C,-0x00083,-0x0008A,-0x00092,-0x00099,
    -0x000A0,-0x000A8,-0x000AF,-0x000B6,-0x000BD,-0x000C3,-0x000C9,-0x000CF,
     0x000D5, 0x000DA, 0x000DE, 0x000E1, 0x000E3, 0x000E4, 0x000E4, 0x000E3,
     0x000E0, 0x000DD, 0x000D7, 0x000D0, 0x000C8, 0x000BD, 0x000B1, 0x000A3,
     0x00092, 0x0007F, 0x0006A, 0x00053, 0x00039, 0x0001D,-0x00001,-0x00023,
    -0x00047,-0x0006E,-0x00098,-0x000C4,-0x000F3,-0x00125,-0x0015A,-0x00190,
    -0x001CA,-0x00206,-0x00244,-0x00284,-0x002C6,-0x0030A,-0x0034F,-0x00396,
    -0x003DE,-0x00427,-0x00470,-0x004B9,-0x00502,-0x0054B,-0x00593,-0x005D9,
    -0x0061E,-0x00661,-0x006A1,-0x006DE,-0x00718,-0x0074D,-0x0077E,-0x007A9,
    -0x007D0,-0x007EF,-0x00808,-0x0081A,-0x00824,-0x00826,-0x0081F,-0x0080E,
     0x007F5, 0x007D0, 0x007A0, 0x00765, 0x0071E, 0x006CB, 0x0066C, 0x005FF,
     0x00586, 0x00500, 0x0046B, 0x003CA, 0x0031A, 0x0025D, 0x00192, 0x000B9,
    -0x0002C,-0x0011F,-0x00220,-0x0032D,-0x00446,-0x0056B,-0x0069B,-0x007D5,
    -0x00919,-0x00A66,-0x00BBB,-0x00D16,-0x00E78,-0x00FDE,-0x01148,-0x012B3,
    -0x01420,-0x0158C,-0x016F6,-0x0185C,-0x019BC,-0x01B16,-0x01C66,-0x01DAC,
    -0x01EE5,-0x02010,-0x0212A,-0x02232,-0x02325,-0x02402,-0x024C7,-0x02570,
    -0x025FE,-0x0266D,-0x026BB,-0x026E6,-0x026ED,-0x026CE,-0x02686,-0x02615,
    -0x02577,-0x024AC,-0x023B2,-0x02287,-0x0212B,-0x01F9B,-0x01DD7,-0x01BDD,
     0x019AE, 0x01747, 0x014A8, 0x011D1, 0x00EC0, 0x00B77, 0x007F5, 0x0043A,
     0x00046,-0x003E5,-0x00849,-0x00CE3,-0x011B4,-0x016B9,-0x01BF1,-0x0215B,
    -0x026F6,-0x02CBE,-0x032B3,-0x038D3,-0x03F1A,-0x04586,-0x04C15,-0x052C4,
    -0x05990,-0x06075,-0x06771,-0x06E80,-0x0759F,-0x07CCA,-0x083FE,-0x08B37,
    -0x09270,-0x099A7,-0x0A0D7,-0x0A7FD,-0x0AF14,-0x0B618,-0x0BD05,-0x0C3D8,
    -0x0CA8C,-0x0D11D,-0x0D789,-0x0DDC9,-0x0E3DC,-0x0E9BD,-0x0EF68,-0x0F4DB,
    -0x0FA12,-0x0FF09,-0x103BD,-0x1082C,-0x10C53,-0x1102E,-0x113BD,-0x116FB,
    -0x119E8,-0x11C82,-0x11EC6,-0x120B3,-0x12248,-0x12385,-0x12467,-0x124EF,
     0x1251E, 0x124F0, 0x12468, 0x12386, 0x12249, 0x120B4, 0x11EC7, 0x11C83,
     0x119E9, 0x116FC, 0x113BE, 0x1102F, 0x10C54, 0x1082D, 0x103BE, 0x0FF0A,
     0x0FA13, 0x0F4DC, 0x0EF69, 0x0E9BE, 0x0E3DD, 0x0DDCA, 0x0D78A, 0x0D11E,
     0x0CA8D, 0x0C3D9, 0x0BD06, 0x0B619, 0x0AF15, 0x0A7FE, 0x0A0D8, 0x099A8,
     0x09271, 0x08B38, 0x083FF, 0x07CCB, 0x075A0, 0x06E81, 0x06772, 0x06076,
     0x05991, 0x052C5, 0x04C16, 0x04587, 0x03F1B, 0x038D4, 0x032B4, 0x02CBF,
     0x026F7, 0x0215C, 0x01BF2, 0x016BA, 0x011B5, 0x00CE4, 0x0084A, 0x003E6,
    -0x00045,-0x00439,-0x007F4,-0x00B76,-0x00EBF,-0x011D0,-0x014A7,-0x01746,
     0x019AE, 0x01BDE, 0x01DD8, 0x01F9C, 0x0212C, 0x02288, 0x023B3, 0x024AD,
     0x02578, 0x02616, 0x02687, 0x026CF, 0x026EE, 0x026E7, 0x026BC, 0x0266E,
     0x025FF, 0x02571, 0x024C8, 0x02403, 0x02326, 0x02233, 0x0212B, 0x02011,
     0x01EE6, 0x01DAD, 0x01C67, 0x01B17, 0x019BD, 0x0185D, 0x016F7, 0x0158D,
     0x01421, 0x012B4, 0x01149, 0x00FDF, 0x00E79, 0x00D17, 0x00BBC, 0x00A67,
     0x0091A, 0x007D6, 0x0069C, 0x0056C, 0x00447, 0x0032E, 0x00221, 0x00120,
     0x0002D,-0x000B8,-0x00191,-0x0025C,-0x00319,-0x003C9,-0x0046A,-0x004FF,
    -0x00585,-0x005FE,-0x0066B,-0x006CA,-0x0071D,-0x00764,-0x0079F,-0x007CF,
     0x007F5, 0x0080F, 0x00820, 0x00827, 0x00825, 0x0081B, 0x00809, 0x007F0,
     0x007D1, 0x007AA, 0x0077F, 0x0074E, 0x00719, 0x006DF, 0x006A2, 0x00662,
     0x0061F, 0x005DA, 0x00594, 0x0054C, 0x00503, 0x004BA, 0x00471, 0x00428,
     0x003DF, 0x00397, 0x00350, 0x0030B, 0x002C7, 0x00285, 0x00245, 0x00207,
     0x001CB, 0x00191, 0x0015B, 0x00126, 0x000F4, 0x000C5, 0x00099, 0x0006F,
     0x00048, 0x00024, 0x00002,-0x0001C,-0x00038,-0x00052,-0x00069,-0x0007E,
    -0x00091,-0x000A2,-0x000B0,-0x000BC,-0x000C7,-0x000CF,-0x000D6,-0x000DC,
    -0x000DF,-0x000E2,-0x000E3,-0x000E3,-0x000E2,-0x000E0,-0x000DD,-0x000D9,
     0x000D5, 0x000D0, 0x000CA, 0x000C4, 0x000BE, 0x000B7, 0x000B0, 0x000A9,
     0x000A1, 0x0009A, 0x00093, 0x0008B, 0x00084, 0x0007D, 0x00075, 0x0006F,
     0x00068, 0x00061, 0x0005B, 0x00055, 0x0004F, 0x00049, 0x00044, 0x0003F,
     0x0003A, 0x00035, 0x00031, 0x0002D, 0x00029, 0x00026, 0x00023, 0x0001F,
     0x0001D, 0x0001A, 0x00018, 0x00015, 0x00013, 0x00011, 0x00010, 0x0000E,
     0x0000D, 0x0000B, 0x0000A, 0x00009, 0x00008, 0x00007, 0x00007, 0x00006,
     0x00005, 0x00005, 0x00004, 0x00004, 0x00003, 0x00003, 0x00002, 0x00002,
     0x00002, 0x00002, 0x00001, 0x00001, 0x00001, 0x00001, 0x00001, 0x00001
];

// Possible quantization per subband

// Quantizer lookup, step 1: bitrate classes
fn QUANT_LUT_STEP1() -> [[i8]/16]/2 {
    [
        // 32, 48, 56, 64, 80, 96,112,128,160,192,224,256,320,384 <- bitrate
        [   0,  0,  1,  1,  1,  2,  2,  2,  2,  2,  2,  2,  2,  2, 0, 0 ],  // mono
        // 16, 24, 28, 32, 40, 48, 56, 64, 80, 96,112,128,160,192 <- BR / chan
        [   0,  0,  0,  0,  0,  0,  1,  1,  1,  2,  2,  2,  2,  2, 0, 0 ]   // stereo
    ]
}

// Quantizer lookup, step 2: bitrate class, sample rate -> B2 table index, sblimit
const QUANT_TAB_A: i8 = 27 | 64;   // high-rate, sblimit = 27
const QUANT_TAB_B: i8 = 30 | 64;   // high-rate, sblimit = 30
const QUANT_TAB_C: i8 = 8;         // low-rate, sblimit = 8
const QUANT_TAB_D: i8 = 12;        // low-rate, sblimit = 12

fn QUANT_LUT_STEP2() -> [[i8]/3]/3 {
    [
        //   44.1 kHz,      48 KHz,       32 kHz,
        [ QUANT_TAB_C, QUANT_TAB_C, QUANT_TAB_D ],  // 32-48 kbit/sec/ch
        [ QUANT_TAB_A, QUANT_TAB_A, QUANT_TAB_A ],  // 56-80 kbit/sec/ch
        [ QUANT_TAB_B, QUANT_TAB_A, QUANT_TAB_B ],  // 96+ kbit/sec/ch
    ]
}

// Quantizer lookup, step 3: B2 table, subband -> nbal, row index
// (Upper 4 bits: nbal, lower 4 bits: row index)
fn QUANT_LUT_STEP3() -> [[i8]/32]/2 {
    [
        // Low-rate table
        [
            0x44,0x44,                                                   // SB  0 -  1
            0x34,0x34,0x34,0x34,0x34,0x34,0x34,0x34,0x34,0x34,           // SB  2 - 12
            0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0                      // Padding
        ],

        // High-rate table
        [
            0x43,0x43,0x43,                                              // SB  0 -  2
            0x42,0x42,0x42,0x42,0x42,0x42,0x42,0x42,                     // SB  3 - 10
            0x31,0x31,0x31,0x31,0x31,0x31,0x31,0x31,0x31,0x31,0x31,0x31, // SB 11 - 22
            0x20,0x20,0x20,0x20,0x20,0x20,0x20,                          // SB 23 - 29
            0,0                                                          // Padding
        ]
    ]
}

// Quantizer lookup, step 4: table row, allocation[] value -> quant table index
fn QUANT_LUT_STEP4() -> [[i8]/16]/5 {
    [
        // 0   1   2   3   4   5   6   7   8   9  10  11  12  13  14  15
        [  0,  1,  2, 17,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0 ],
        [  0,  1,  2,  3,  4,  5,  6, 17,  0,  0,  0,  0,  0,  0,  0,  0 ],
        [  0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 17 ],
        [  0,  1,  3,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15, 16, 17 ],
        [  0,  1,  2,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15, 17 ]
    ]
}

// Quantizer specification structure
struct QuantizerSpec {
    nlevels: u16;
    grouping: i8;
    cw_bits: i8;
    Smul: u16;
    Sdiv: u16;
}

fn QuantizerSpec(nlevels: u16, grouping: i8, cw_bits: i8, Smul: u16, Sdiv: u16) -> QuantizerSpec {
    QuantizerSpec {
        nlevels: nlevels,
        grouping: grouping,
        cw_bits: cw_bits,
        Smul: Smul,
        Sdiv: Sdiv
    }
}

// Quantizer table
fn QUANTIZER_TABLE() -> [QuantizerSpec]/17 {
    [
        QuantizerSpec(    3, 1,  5, 0x7FFF, 0xFFFF),
        QuantizerSpec(    5, 1,  7, 0x3FFF, 0x0002),
        QuantizerSpec(    7, 0,  3, 0x2AAA, 0x0003),
        QuantizerSpec(    9, 1, 10, 0x1FFF, 0x0002),
        QuantizerSpec(   15, 0,  4, 0x1249, 0xFFFF),
        QuantizerSpec(   31, 0,  5, 0x0888, 0x0003),
        QuantizerSpec(   63, 0,  6, 0x0421, 0xFFFF),
        QuantizerSpec(  127, 0,  7, 0x0208, 0x0009),
        QuantizerSpec(  255, 0,  8, 0x0102, 0x007F),
        QuantizerSpec(  511, 0,  9, 0x0080, 0x0002),
        QuantizerSpec( 1023, 0, 10, 0x0040, 0x0009),
        QuantizerSpec( 2047, 0, 11, 0x0020, 0x0021),
        QuantizerSpec( 4095, 0, 12, 0x0010, 0x0089),
        QuantizerSpec( 8191, 0, 13, 0x0008, 0x0249),
        QuantizerSpec(16383, 0, 14, 0x0004, 0x0AAB),
        QuantizerSpec(32767, 0, 15, 0x0002, 0x3FFF),
        QuantizerSpec(65535, 0, 16, 0x0001, 0xFFFF)
    ]
}

// Workaround for the fact that some constants are unimplemented in Rust.
struct MP2Constants {
    QUANT_LUT_STEP1: [[i8]/16]/2;
    QUANT_LUT_STEP2: [[i8]/3]/3;
    QUANT_LUT_STEP3: [[i8]/32]/2;
    QUANT_LUT_STEP4: [[i8]/16]/5;
    QUANTIZER_TABLE: [QuantizerSpec]/17;
}

fn MP2Constants() -> MP2Constants {
    MP2Constants {
        QUANT_LUT_STEP1: QUANT_LUT_STEP1(),
        QUANT_LUT_STEP2: QUANT_LUT_STEP2(),
        QUANT_LUT_STEP3: QUANT_LUT_STEP3(),
        QUANT_LUT_STEP4: QUANT_LUT_STEP4(),
        QUANTIZER_TABLE: QUANTIZER_TABLE()
    }
}

// Initialization

// FIXME: Eventually these constants should all become "const" values of various sorts, but the
// Rust compiler doesn't support all of them yet.

struct MP2Context {
    constants: MP2Constants;
    N: [[mut i32]/32]/64;
}

fn MP2Context() -> MP2Context {
    let N = [ [ mut 0, ..32 ], ..64 ];
    for range(0, 64) |i| {
        for range(0, 32) |j| {
            N[i][j] = (256.0 * cos(((16+i) * ((j<<1)+1)) as float * 0.0490873852123405)) as i32;
        }
    };

    return MP2Context { constants: MP2Constants(), N: N };
}

struct MP2Stream {
    context: &MP2Context;
    V: [[mut i32]/1024]/2;
    mut Voffs: i32;
    U: [mut i32]/512;
}

fn MP2Stream(context: &MP2Context) -> MP2Stream {
    MP2Stream {
        context: context,
        V: [ [ mut 0, ..1024 ], [ mut 0, ..1024 ] ],
        Voffs: 0,
        U: [ mut 0, ..512 ]
    }
}

// Bitstream reading

struct Bitstream {
    mut bit_window: i32;
    mut bits_in_window: i32;
    mut frame_pos: &[u8];
}

impl Bitstream {
    fn show_bits(bit_count: i32) -> i32 {
        self.bit_window >> (24 - bit_count)
    }

    fn get_bits(bit_count: i32) -> i32 {
        let result = self.show_bits(bit_count);
        self.bit_window = (self.bit_window << bit_count) & 0xffffff;
        self.bits_in_window -= bit_count;
        while self.bits_in_window < 16 {
            let ch = self.frame_pos[0];
            self.frame_pos = view(self.frame_pos, 1, self.frame_pos.len());
            self.bit_window |= (ch as i32) << (16 - self.bits_in_window);
            self.bits_in_window += 8;
        }
        return result;
    }
}

// Frame decoding

impl MP2Stream {
    // Helper functions

    fn read_allocation(bitstream: Bitstream, sb: i32, b2_table: i32)
                    -> option<&self/QuantizerSpec> {
        let table_idx = self.context.constants.QUANT_LUT_STEP3[b2_table][sb] as i32;
        let bits = bitstream.get_bits(table_idx >> 4);
        let table_idx = self.context.constants.QUANT_LUT_STEP4[table_idx & 15][bits];
        if table_idx != 0 {
            return Some(&self.context.constants.QUANTIZER_TABLE[table_idx - 1]);
        }
        return None;
    }

    fn read_samples(bitstream: Bitstream, q_opt: option<&self/QuantizerSpec>, scalefactor: i32,
                    sample: &[mut i32]) {
        let q;
        match q_opt {
            None => {
                // No bits allocated for this sub-band.
                sample[0] = 0;
                sample[1] = 0;
                sample[2] = 0;
                return;
            }
            Some(quantizer) => {
                q = quantizer;
            }
        }

        // Resolve the scale factor.
        let scalefactor = SCF_VALUE[scalefactor];

        // Decode samples.
        let mut adj = q.nlevels as i32;
        if q.grouping != 0 {
            // Decode grouped samples.
            let mut val = bitstream.get_bits(q.cw_bits as i32);
            sample[0] = val % adj;
            val /= adj;
            sample[1] = val % adj;
            sample[2] = val / adj;
        } else {
            // Decode direct samples.
            for range(0, 3) |idx| {
                sample[idx] = bitstream.get_bits(q.cw_bits as i32);
            }
        }

        // Postmultiply samples.
        adj = ((adj + 1) >> 1) - 1;
        for range(0, 3) |idx| {
            // Step 1: Renormalization to [-1..1].
            let mut val = adj - (sample[idx] as i32);
            val = (val * (q.Smul as i32)) + (val / (q.Sdiv as i32));
            // Step 2: Apply scale factor.
            sample[idx] = (val * (scalefactor >> 12) +                       // Upper part
                           ((val * (scalefactor & 4095) + 2048) >> 12)) >>   // Lower part
                           12;                                               // Scale adjust
        }
    }

    // Main functions

    fn get_sample_rate(frame: &[u8]) -> MP2Result<i32> {
        if frame[0] != 0xff {
            return Error("no valid syncword");
        }
        if frame[1] != 0xfd {
            return Error("not MPEG-1 Audio Layer II without redundancy");
        }
        if (frame[2] - 0x10) >= 0xe0 {
            return Error("invalid bitrate");
        }
        return OK(SAMPLE_RATES[(frame[2] >> 2) & 3]);
    }

    fn decode_frame(frame: &[u8], pcm: &[mut i16]) -> MP2Result<i32> {
        let mut pcm = pcm;

        // Check for valid header; syncword OK, MPEG-Audio Layer II
        if frame[0] != 0xff || (frame[1] & 0xfe) != 0xfc {
            return Error("invalid MPEG-Audio Layer II header");
        }

        // Set up the bitstream reader.
        let bitstream = Bitstream {
            bit_window: (frame[2] as i32) << 16,
            bits_in_window: 8,
            frame_pos: view(frame, 3, frame.len())
        };

        // Read the rest of the header.
        let bit_rate_index_minus1 = bitstream.get_bits(4) - 1;
        if bit_rate_index_minus1 > 13 {
            return Error("invalid bit rate or 'free format'");
        }
        let sampling_frequency = bitstream.get_bits(2);
        if sampling_frequency == 3 {
            return Error("invalid sampling frequency");
        }
        let padding_bit = bitstream.get_bits(1);
        ignore(bitstream.get_bits(1));  // Discard the private bit.
        let mode = Mode(bitstream.get_bits(2));

        // Parse the mode extension; set up the stereo bound.
        let mut bound;
        match mode {
            JointStereo => {
                bound = (bitstream.get_bits(2) + 1) << 2;
            }
            Mono => {
                ignore(bitstream.get_bits(2));
                bound = 0;
            }
            Stereo | DualChannel => {
                ignore(bitstream.get_bits(2));
                bound = 32;
            }
        }

        // Discard the last 4 bits of the header and the CRC value if present.
        ignore(bitstream.get_bits(4));
        if (frame[1] & 1) == 0 {
            ignore(bitstream.get_bits(16));
        }

        // Compute the frame size.
        let mut frame_size = 144000 * BITRATES[bit_rate_index_minus1];
        frame_size /= SAMPLE_RATES[sampling_frequency];
        frame_size += padding_bit;
        if pcm.len() < (frame_size as uint) {
            return Error("PCM too small");
        }

        // Prepare the quantizer table lookups.
        let mut table_idx = if mode == Mono { 0 } else { 1 };
        let QUANT_LUT_STEP1 = &self.context.constants.QUANT_LUT_STEP1;
        let QUANT_LUT_STEP2 = &self.context.constants.QUANT_LUT_STEP2;
        table_idx = QUANT_LUT_STEP1[table_idx][bit_rate_index_minus1] as i32;
        table_idx = QUANT_LUT_STEP2[table_idx][sampling_frequency] as i32;
        let sblimit = table_idx & 63;
        table_idx >>= 6;
        if bound > sblimit {
            bound = sblimit;
        }

        // Read the allocation information.
        let allocation = [ [ mut None, ..32 ], [ mut None, ..32 ] ];
        let num_channels = if mode == Mono { 1 } else { 2 };
        for range(0, bound) |sb| {
            for range(0, 2) |ch| {
                allocation[ch][sb] = self.read_allocation(bitstream, sb as i32, table_idx);
            }
        }
        for range(bound, sblimit) |sb| {
            let alloc = self.read_allocation(bitstream, sb as i32, table_idx);
            allocation[0][sb] = alloc;
            allocation[1][sb] = alloc;
        }

        // Read scale factor selector information.
        let scfsi = [ [ mut 0, ..32 ], [ mut 0, ..32 ] ];
        for range(0, sblimit) |sb| {
            for range(0, num_channels) |ch| {
                if allocation[ch][sb].is_some() {
                    scfsi[ch][sb] = bitstream.get_bits(2);
                }
            }
            if mode == Mono {
                scfsi[1][sb] = scfsi[0][sb];
            }
        }

        // Read scale factors.
        let scalefactor = [ [ [ mut 0, 0, 0 ], ..32 ], [ [ mut 0, 0, 0 ], ..32 ] ];
        for range(0, sblimit) |sb| {
            for range(0, num_channels) |ch| {
                if allocation[ch][sb].is_some() {
                    match scfsi[ch][sb] {
                        0 => {
                            scalefactor[ch][sb][0] = bitstream.get_bits(6);
                            scalefactor[ch][sb][1] = bitstream.get_bits(6);
                            scalefactor[ch][sb][2] = bitstream.get_bits(6);
                        }
                        1 => {
                            let a = bitstream.get_bits(6);
                            scalefactor[ch][sb][0] = a;
                            scalefactor[ch][sb][1] = a;
                            scalefactor[ch][sb][2] = bitstream.get_bits(6);
                        }
                        2 => {
                            let a = bitstream.get_bits(6);
                            scalefactor[ch][sb][0] = a;
                            scalefactor[ch][sb][1] = a;
                            scalefactor[ch][sb][2] = a;
                        }
                        3 => {
                            scalefactor[ch][sb][0] = bitstream.get_bits(6);
                            let a = bitstream.get_bits(6);
                            scalefactor[ch][sb][1] = a;
                            scalefactor[ch][sb][2] = a;
                        }
                        _ => fail
                    }
                }
            }
            if mode == Mono {
                for range(0, 3) |part| {
                    scalefactor[1][sb][part] = scalefactor[0][sb][part];
                }
            }
        }

        // Perform coefficient input and reconstruction.
        let sample = [ [ [ mut 0, 0, 0 ], ..32 ], [ [ mut 0, 0, 0 ], ..32 ] ];
        for range(0, 3) |part| {    // For each part...
            for 4.times {           // For each granule...
                // Read the samples.
                for range(0, bound) |sb| {
                    for range(0, 2) |ch| {
                        self.read_samples(bitstream, allocation[ch][sb], scalefactor[ch][sb][part],
                                          sample[ch][sb]);
                    }
                }
                for range(bound, sblimit) |sb| {
                    self.read_samples(bitstream, allocation[0][sb], scalefactor[0][sb][part],
                                      sample[0][sb]);
                    for range(0, 3) |idx| {
                        sample[1][sb][idx] = sample[0][sb][idx];
                    }
                }
                for range(0, 2) |ch| {
                    for range(sblimit, 32) |sb| {
                        for range(0, 3) |idx| {
                            sample[ch][sb][idx] = 0;
                        }
                    }
                }

                // Synthesis loop
                for range(0, 3) |idx| {
                    // Shifting step
                    table_idx = (self.Voffs - 64) & 1023;
                    self.Voffs = table_idx;

                    for range(0, 2) |ch| {
                        // Matrixing
                        for range(0, 64) |i| {
                            let i = i as i32;
                            let mut sum = 0;
                            for range(0, 32) |j| {
                                sum += self.context.N[i][j] * sample[ch][j][idx]; // 8b * 15b = 23b
                            }
                            // Intermediate value is 28-bit (23 + 5), clamp to 14 bit.
                            self.V[ch][table_idx + i] = (sum + 8192) >> 14;
                        }

                        // Construction of U
                        for range(0, 8) |i| {
                            for range(0, 32) |j| {
                                self.U[(i<<6)+j]    = self.V[ch][(table_idx+(i<<7)+j)    & 1023];
                                self.U[(i<<6)+j+32] = self.V[ch][(table_idx+(i<<7)+j+96) & 1023];
                            }
                        }

                        // Apply window.
                        for range(0, 512) |i| {
                            self.U[i] = (self.U[i] * D[i] + 32) >> 6;
                        }

                        // Output samples.
                        for range(0, 32) |j| {
                            let mut sum: i32 = 0;
                            for range(0, 16) |i| {
                                sum -= self.U[(i << 5) + j];
                            }
                            sum = (sum + 8) >> 4;
                            if sum < -32768 {
                                sum = -32768;
                            }
                            if sum > 32767 {
                                sum = 32767;
                            }
                            pcm[(idx << 6) | (j << 1) | ch] = sum as i16;
                        }
                    }   // End of synthesis channel loop.
                }   // End of synthesis sub-block loop.

                // Adjust PCM output slice: decoded 3 * 32 = 96 stereo samples.
                pcm = mut_view(pcm, 192, pcm.len());
            }
        }

        return OK(frame_size);
    }
}

// Entry point

fn main(args: ~[UniqueString]) {
    if args.len() < 2 {
        println(fmt!("usage: %s file.mp2", args[0]));
        return;
    }

    let result = io::read_whole_file(args[1]);
    let bytes = match result {
        OK(_)    => unwrap(result),
        Error(e) => { println(e); return; }
    };
    let mut bytes = view(bytes, 0, bytes.len());

    let context = MP2Context();
    let stream = MP2Stream(&context);
    let sample_rate = stream.get_sample_rate(bytes).get() as int;
    println(fmt!("sample rate is %d", sample_rate));

    let ao = ao::AO();
    let sample_format = ao::SampleFormat(16, sample_rate as i32, 2, ao::Little);
    let device = ao.open_live(ao.default_driver_id(), &sample_format);

    let pcm = [ mut 0, ..2304 ];    // FIXME: Rust compiler should accept (SAMPLES_PER_FRAME*2).
    loop {
        let result = stream.decode_frame(bytes, pcm);
        let frame_size;
        match result {
            OK(size) => frame_size = size as uint,
            Error(e) => { println(from_slice(e)); return; }
        }

        // Write the bytes, in little-endian.
        device.play(pcm);

        if bytes.len() < frame_size { return; }
        bytes = view(bytes, frame_size, bytes.len());
    }
}

