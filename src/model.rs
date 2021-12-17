use bincode::enc::write::Writer;
use bincode::{
    config::Configuration, de::Decoder, enc::Encoder, error::DecodeError, error::EncodeError,
    Decode, Encode,
};
use std::default::Default;

#[derive(Encode, Decode, PartialEq, Debug, Clone, Default)]
pub struct Job {
    pub from: u32,
    pub to: u32,
    pub header: Blob, //ByteString hash
    pub txs: Blob,    //ByteString hash
    pub target: Blob, //BigInteger
}

type Jobs = Vec<Job>;
type Blob = Vec<u8>;

#[derive(Encode, Decode, PartialEq, Debug, Clone, Default)]
pub struct SubmitResult {
    from: u32,
    to: u32,
    status: bool,
}

#[derive(Debug)]
pub enum Body {
    Jobs(Jobs),
    SubmitResult(SubmitResult),
}

pub enum WorkUnit {
    Job(Job),
    SubmitRes(String, SubmitResult),
}

pub struct SubmitReq {
    pub nonce: Vec<u8>, //len = 24
    pub header: Blob,   //ByteString hash
    pub txs: Blob,      //ByteString hash
}

impl Default for Body {
    fn default() -> Self {
        Body::Jobs(Jobs::default())
    }
}

impl Clone for Body {
    #[must_use = "cloning is often expensive and is not expected to have side effects"]
    fn clone(&self) -> Self {
        match self {
            Body::Jobs(job) => Body::Jobs(job.clone()),
            Body::SubmitResult(ref ret) => Body::SubmitResult(ret.clone()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Message {
    len: u32, //len = len(kind) + len(body)[👌];len = len + len(kind) + len(body)[🤯];
    kind: u8, //1 = Jobs ; 0 = SubmitResult
    body: Body,
}

impl Message {
    fn body(body: Body) -> Message {
        let mut len: u32 = 0;
        let mut kind: u8 = 0;
        let option = bincode::config::Configuration::standard()
            .with_big_endian()
            .with_no_limit()
            .write_fixed_array_length()
            .with_fixed_int_encoding();
        let mut size = 4 + 1;
        match &body {
            Body::Jobs(jobs) => {
                let body = bincode::encode_to_vec(jobs, option).unwrap();
            }
            Body::SubmitResult(ret) => {}
        }
        Message { len, kind, body }
    }

    pub fn job(job: Job) -> Self {
        Message::body(Body::Jobs(vec![job]))
    }
}

impl Encode for Message {
    /// Encode a given type.
    fn encode<E: Encoder>(&self, mut encoder: E) -> Result<(), EncodeError> {
        // let mut size:u32 = 4 + 1 ; //字节数?
        let mut size: u32 = 1; //字节数
        let option = bincode::config::Configuration::standard()
            .with_big_endian()
            .with_no_limit()
            .with_fixed_int_encoding();
        match &self.body {
            Body::Jobs(ref jobs) => {
                let body = bincode::encode_to_vec(jobs, option)?;
                // let mut total:u32= 0;
                // for job in jobs {
                //     total += (4 * 5  + job.target.len() + job.header.len() + job.txs.len()) as u32;
                // }
                // println!("total := {:?}",total);
                // println!("body.len() := {:?}",body.len());
                size += (body.len() as u32);
                (size as u32).encode(&mut encoder)?;
                (0 as u8).encode(&mut encoder)?;
                jobs.encode(encoder)
            }
            Body::SubmitResult(ret) => {
                size += (4 + 4 + 1);
                // let len = std::mem::size_of::<SubmitResult>();
                // size += (len / 8) as u32;
                (size as u32).encode(&mut encoder)?;
                (1 as u8).encode(&mut encoder)?;
                // (len as u32).encode(&mut encoder)?;
                ret.encode(encoder)
            }
        }
    }
}

impl Decode for Message {
    fn decode<D: Decoder>(mut decoder: D) -> Result<Self, DecodeError> {
        let size = u32::decode(&mut decoder)?;
        let kind = u8::decode(&mut decoder)?;
        if kind == 0 {
            let job = Jobs::decode(&mut decoder)?;
            Ok(Message {
                len: size,
                kind,
                body: Body::Jobs(job),
            })
        } else {
            let ret = SubmitResult::decode(&mut decoder)?;
            Ok(Message {
                len: size,
                kind,
                body: Body::SubmitResult(ret),
            })
        }
    }
}

// SubmitResult
// [0, 0, 0, 10,
//  1,
//  0, 0, 0, 0,
//  0, 0, 0, 1,
//  1
// ]

// 传输协议:
// message_size(4 字节) + kind（1字节） + data
// message_size: 4 字节；整形。
// kind:0 = JOBS or 1 = SUBMIT_RESULT;
// data: bolb | submit_result_t
// submit_result_t: from(4字节) + to(4字节) + status(1字节)
// *_blob: len(4字节) + vec<u8>
// job:  from(4字节) + to(4字节) + header_blob + txs_blob + target_blob
// jobs: len(4字节) + job[*]

//kind: 种类，结构体类型。
//结构体：len + data。
//bool: 1个字节。
//字符串：len(4字节) + data(数据)。
//总数据长度：total >= 4。
//按结构体顺序拼接字符。
//VarintEncoding 动态类型：len(4字节) + data(bytes)
//Fixed type:size::of(val)
//vec<T> 类型是动态的

// broker_ip = 127.0.0.1, port = 10973

#[cfg(test)]
mod tests {
    use super::{Body, Message, SubmitResult};
    use crate::model::Job;
    use std::string::String;

    #[test]
    fn test_basic_struct() {
        #[derive(Encode, Decode, PartialEq, Debug)]
        struct Easy {
            x: isize,
            y: usize,
            z: i8,
            k: u64,
            kkk: String,
        }

        let easy = &Easy {
            x: 1,
            y: 2,
            z: 1,
            k: 123,
            kkk: String::from("ab"),
        };
        let option = bincode::config::Configuration::standard()
            .with_big_endian()
            .with_no_limit()
            .with_fixed_int_encoding();
        let data = bincode::encode_to_vec(easy, option).expect("encode_to_vec msg error");
        println!("{:?}", data.len());
        println!("{:?}", data);
        let hex_server_message1 = "0000000a01000000000000000101";
        let decoded = hex::decode(hex_server_message1).expect("hex decode msg error");
        println!("{:?}", decoded);
        let res = bincode::decode_from_slice::<Message, _>(decoded.as_slice(), option);
        if let Ok(val) = res {
            if let Body::SubmitResult(ref ret) = val.0.body {
                assert_eq!(ret.status, true);
            }
            let server_message0 = bincode::encode_to_vec(val.0.clone(), option);
            assert_eq!(
                server_message0.expect("server_message0_0 == decoded"),
                decoded
            );
        }

        let hex_server_message0 = "00001af5000000001000000000000000000000012e00070000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec411500001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f00001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c000002349b6af9967de46fc5e7cce9858fb8a7b7ef334e194ab2a040b21e511b100000225e862ddfed99cda3f859a01758a58c90876c703676e0dccb35951f97200001832f64f200d39da63f58d059d062ba6ce7df1218a0af6a1ad6cb859a1334a454d47afc7672488d906f7699d6a2f73fae33c265afc6d9eda3b60351cd2fa60cd823ebdd1605cd2ff4a80c3540dd93ad20a9957519d4dc9612052dc4ed11b0000017bf884c7a71e2456710000004f01010080004e20bb9aca000001c41a055690d9db800000627ae790cbf98235030992faf3496d94da8900ba1dae0ee3458b10a20b633e500000017bf88def67000a00000000017bf884c7a7000000000000001e24567100000000000000000000000000000000000000000000000000000000000000000000010000012e00070000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec411500001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f00001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c000002349b6af9967de46fc5e7cce9858fb8a7b7ef334e194ab2a040b21e511b100000225e862ddfed99cda3f859a01758a58c90876c703676e0dccb35951f97200001832f64f200d39da63f58d059d062ba6ce7df1218a0af6a1ad6cb859a1334a454d47afc7672488d906f7699d6a2f73fae33c265afc6d9eda3b60351cd2fadc52ede9d0afacaddbecff9f1925e08e7bba59556dabfcb93f60aa9d218d6a340000017bf884c7b11e25f54d0000004f01010080004e20bb9aca000001c41a055690d9db800000303b30e2c4379f0bd813e60f86b412f21ac92c8ba9012e79da61ff714fe166fd0000017bf88def71000a00010000017bf884c7b1000000000000001e25f54d00000000000000000000000000000000000000000000000000000000000000000000020000012e00070000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec411500001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f00001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c000002349b6af9967de46fc5e7cce9858fb8a7b7ef334e194ab2a040b21e511b100000225e862ddfed99cda3f859a01758a58c90876c703676e0dccb35951f97200001832f64f200d39da63f58d059d062ba6ce7df1218a0af6a1ad6cb859a1334a454d47afc7672488d906f7699d6a2f73fae33c265afc6d9eda3b60351cd2faa93b9a5754e2ce79b5bc3bc34fa54e12ed01293eebb11090a859d401ae93c5a70000017bf884c7b81e27eca80000004f01010080004e20bb9aca000001c41a055690d9db80000047d375949b85b11ffd5385518183926ac12df8a77b5e768a11b4d7ca44aa71e20000017bf88def78000a00020000017bf884c7b8000000000000001e27eca800000000000000000000000000000000000000000000000000000000000000000000030000012e00070000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec411500001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f00001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c000002349b6af9967de46fc5e7cce9858fb8a7b7ef334e194ab2a040b21e511b100000225e862ddfed99cda3f859a01758a58c90876c703676e0dccb35951f97200001832f64f200d39da63f58d059d062ba6ce7df1218a0af6a1ad6cb859a1334a454d47afc7672488d906f7699d6a2f73fae33c265afc6d9eda3b60351cd2fa86188b14528c7b14eb989c0b4b39c32b0db8c63a31702096c261bc7ebc02225d0000017bf884c7bb1e25531e0000004f01010080004e20bb9aca000001c41a055690d9db80000097e542bd01a14e12b1a8076e55acdcdfe4704eb297ace191402b5a58cb81016f0000017bf88def7b000a00030000017bf884c7bb000000000000001e25531e00000000000000000000000000000000000000000000000000000000000001000000000000012e000700001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c000001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f00001503034b6a8a3906abe80b33dab6f7311339fd2130df61f01e2e43319cc40000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec41150000111035bb3104c52f7886fc297522dc434a0898ea3c89a0fced320b45c3160000170a4d2327a15f77e5ed8b17f422b711376d09969282bb98065af15d26576c2d43c7d78c8f904c9a89b433cddec61874044cb95591a112c15511b487a7bf88a12da4b0617df1ce801186250f0b7ac213334b1e258ce25c8a40d9ac48f6c80000017bf884c7c01e21af500000004f01010080004e20bb9aca000001c41a055690d9db800000627ae790cbf98235030992faf3496d94da8900ba1dae0ee3458b10a20b633e500000017bf88def80000a01000000017bf884c7c0000000000000001e21af5000000000000000000000000000000000000000000000000000000000000001000000010000012e000700001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c000001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f00001503034b6a8a3906abe80b33dab6f7311339fd2130df61f01e2e43319cc40000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec41150000111035bb3104c52f7886fc297522dc434a0898ea3c89a0fced320b45c3160000170a4d2327a15f77e5ed8b17f422b711376d09969282bb98065af15d26576c2d43c7d78c8f904c9a89b433cddec61874044cb95591a112c15511b487a7bff35a7b57892451169c449661b99dd9d81b2090511c17f74d13b59e6575ef5a260000017bf884c7c11e2330de0000004f01010080004e20bb9aca000001c41a055690d9db800000303b30e2c4379f0bd813e60f86b412f21ac92c8ba9012e79da61ff714fe166fd0000017bf88def81000a01010000017bf884c7c1000000000000001e2330de00000000000000000000000000000000000000000000000000000000000001000000020000012e000700001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c000001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f00001503034b6a8a3906abe80b33dab6f7311339fd2130df61f01e2e43319cc40000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec41150000111035bb3104c52f7886fc297522dc434a0898ea3c89a0fced320b45c3160000170a4d2327a15f77e5ed8b17f422b711376d09969282bb98065af15d26576c2d43c7d78c8f904c9a89b433cddec61874044cb95591a112c15511b487a7bffe7810d753243282941e09b840edb58a34072b71a1bc9c3676976e75d59580490000017bf884c7c31e241e5d0000004f01010080004e20bb9aca000001c41a055690d9db80000047d375949b85b11ffd5385518183926ac12df8a77b5e768a11b4d7ca44aa71e20000017bf88def83000a01020000017bf884c7c3000000000000001e241e5d00000000000000000000000000000000000000000000000000000000000001000000030000012e000700001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c000001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f00001503034b6a8a3906abe80b33dab6f7311339fd2130df61f01e2e43319cc40000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec41150000111035bb3104c52f7886fc297522dc434a0898ea3c89a0fced320b45c3160000170a4d2327a15f77e5ed8b17f422b711376d09969282bb98065af15d26576c2d43c7d78c8f904c9a89b433cddec61874044cb95591a112c15511b487a7bff903273a7106fd99f20a3ba9124a36caa71297b8a9e5d318eb3fbff66ec90fb60000017bf884c7c41e2100610000004f01010080004e20bb9aca000001c41a055690d9db80000097e542bd01a14e12b1a8076e55acdcdfe4704eb297ace191402b5a58cb81016f0000017bf88def84000a01030000017bf884c7c4000000000000001e21006100000000000000000000000000000000000000000000000000000000000002000000000000012e000700001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c00000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec4115000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f0000179daff6fb7468cdf1ee72dd94f3912e33786490dcc550c3e77e644fa22800001c16be82a2a19e26def1300afe7b19881463c255664f0ce1ec8a982eb6f900001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000010f1715760c2be3563a5e34bd3b6155e6f6f13e39a8056f46f3bf34ab24b0c23cfe2589f2ece18830f0c9dd6e07db135efc1f3adf88c2111e90a3ef7c08b9e9ff0d009a864419f6c56449d1be704f3f7ffd0909303db506844ec5d763b610000017bf884c7c51e26da660000004f01010080004e20bb9aca000001c41a055690d9db800000627ae790cbf98235030992faf3496d94da8900ba1dae0ee3458b10a20b633e500000017bf88def85000a02000000017bf884c7c5000000000000001e26da6600000000000000000000000000000000000000000000000000000000000002000000010000012e000700001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c00000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec4115000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f0000179daff6fb7468cdf1ee72dd94f3912e33786490dcc550c3e77e644fa22800001c16be82a2a19e26def1300afe7b19881463c255664f0ce1ec8a982eb6f900001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000010f1715760c2be3563a5e34bd3b6155e6f6f13e39a8056f46f3bf34ab24b0c23cfe2589f2ece18830f0c9dd6e07db135efc1f3adf88c2111e90a3ef7c08b41812cddacd14fe1dd2fba9bffb4e1d89e90ef5ec3b5eb5b5ed1a45e4bad90880000017bf884c7c51e22dd680000004f01010080004e20bb9aca000001c41a055690d9db800000303b30e2c4379f0bd813e60f86b412f21ac92c8ba9012e79da61ff714fe166fd0000017bf88def85000a02010000017bf884c7c5000000000000001e22dd6800000000000000000000000000000000000000000000000000000000000002000000020000012e000700001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c00000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec4115000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f0000179daff6fb7468cdf1ee72dd94f3912e33786490dcc550c3e77e644fa22800001c16be82a2a19e26def1300afe7b19881463c255664f0ce1ec8a982eb6f900001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000010f1715760c2be3563a5e34bd3b6155e6f6f13e39a8056f46f3bf34ab24b0c23cfe2589f2ece18830f0c9dd6e07db135efc1f3adf88c2111e90a3ef7c08b7f1080cba230811d2a0b28149534cc0f62ff18f4b8c56e62e7b55a06a36c1c070000017bf884c7ce1e2322e10000004f01010080004e20bb9aca000001c41a055690d9db80000047d375949b85b11ffd5385518183926ac12df8a77b5e768a11b4d7ca44aa71e20000017bf88def8e000a02020000017bf884c7ce000000000000001e2322e100000000000000000000000000000000000000000000000000000000000002000000030000012e000700001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c00000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec4115000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f0000179daff6fb7468cdf1ee72dd94f3912e33786490dcc550c3e77e644fa22800001c16be82a2a19e26def1300afe7b19881463c255664f0ce1ec8a982eb6f900001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000010f1715760c2be3563a5e34bd3b6155e6f6f13e39a8056f46f3bf34ab24b0c23cfe2589f2ece18830f0c9dd6e07db135efc1f3adf88c2111e90a3ef7c08b14bca80748f00984d765e578b1fdc20312cb5d26eba70cc279c64fe3e096b43f0000017bf884c7d01e22e1700000004f01010080004e20bb9aca000001c41a055690d9db80000097e542bd01a14e12b1a8076e55acdcdfe4704eb297ace191402b5a58cb81016f0000017bf88def90000a02030000017bf884c7d0000000000000001e22e17000000000000000000000000000000000000000000000000000000000000003000000000000012e000700001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c00000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec411500001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000007f5ce0dcddcfd21ba899e4988227315aec1b303b0a68653c3c6f7b843ac00001a557a4e95d0ce810bef592d9b81596da5b43220699a0339c0d0cf21958d000018d15bf8dc5f105110e2a8053d9ce0b681a62f61e742f2f71680c05a105e000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f9518f6ccc3e057e886ffddf524bb0687a203dfc8ca7db6e23001bf1c8eab10411fd91418568a8b856af4609e4dfa376e32803e2d7870d489b2a2d444a254b9160000017bf884c7d31e278c480000004f01010080004e20bb9aca000001c41a055690d9db800000627ae790cbf98235030992faf3496d94da8900ba1dae0ee3458b10a20b633e500000017bf88def93000a03000000017bf884c7d3000000000000001e278c4800000000000000000000000000000000000000000000000000000000000003000000010000012e000700001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c00000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec411500001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000007f5ce0dcddcfd21ba899e4988227315aec1b303b0a68653c3c6f7b843ac00001a557a4e95d0ce810bef592d9b81596da5b43220699a0339c0d0cf21958d000018d15bf8dc5f105110e2a8053d9ce0b681a62f61e742f2f71680c05a105e000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f9518f6ccc3e057e886ffddf524bb0687a203dfc8ca7db6e23001bf1c8eab1041d89486ac231f0c3df2be758e5d3f95ded505de69a3e98c5fcb6d0c5453f67f6c0000017bf884c7d31e24a7ae0000004f01010080004e20bb9aca000001c41a055690d9db800000303b30e2c4379f0bd813e60f86b412f21ac92c8ba9012e79da61ff714fe166fd0000017bf88def93000a03010000017bf884c7d3000000000000001e24a7ae00000000000000000000000000000000000000000000000000000000000003000000020000012e000700001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c00000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec411500001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000007f5ce0dcddcfd21ba899e4988227315aec1b303b0a68653c3c6f7b843ac00001a557a4e95d0ce810bef592d9b81596da5b43220699a0339c0d0cf21958d000018d15bf8dc5f105110e2a8053d9ce0b681a62f61e742f2f71680c05a105e000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f9518f6ccc3e057e886ffddf524bb0687a203dfc8ca7db6e23001bf1c8eab1041c90c1f5cebc84972c0d515a4aec63420aac1be0a62f902f385d4c18c0f8154ff0000017bf884c7d51e250f0b0000004f01010080004e20bb9aca000001c41a055690d9db80000047d375949b85b11ffd5385518183926ac12df8a77b5e768a11b4d7ca44aa71e20000017bf88def95000a03020000017bf884c7d5000000000000001e250f0b00000000000000000000000000000000000000000000000000000000000003000000030000012e000700001e4218bfc7f7126f97192c3ae193ec5526eab4f0edb090246216f1aa80c00000161fe401202e1345d2d2f38d57dd5715f7f54953cde71b2d7ccd45ec411500001b25550e7be2d69b5f764ba833a02943a656605b320e79a6dc4dda88924a000007f5ce0dcddcfd21ba899e4988227315aec1b303b0a68653c3c6f7b843ac00001a557a4e95d0ce810bef592d9b81596da5b43220699a0339c0d0cf21958d000018d15bf8dc5f105110e2a8053d9ce0b681a62f61e742f2f71680c05a105e000009756a21e45136379b925bf024d9e6d4134a92bede7be5bc32801c2ae08f9518f6ccc3e057e886ffddf524bb0687a203dfc8ca7db6e23001bf1c8eab1041385cfb0633b1f0728b648375d7982368d76e3f1cf4ac4a4d3f94ebdb59a40edd0000017bf884c7d91e22b1ca0000004f01010080004e20bb9aca000001c41a055690d9db80000097e542bd01a14e12b1a8076e55acdcdfe4704eb297ace191402b5a58cb81016f0000017bf88def99000a03030000017bf884c7d9000000000000001e22b1ca000000000000000000000000000000000000000000000000000000";
        let decoded = hex::decode(hex_server_message0).expect("hex decode msg error");
        // println!("{:?}", decoded_string);
        let res = bincode::decode_from_slice::<Message, _>(decoded.as_slice(), option);
        // println!("{:?}", res);
        if let Ok(val) = res {
            if let Body::Jobs(ref ret) = val.0.body {
                assert_eq!(ret.len(), 16);
                let data = ret.clone().split_off(15);
                let mut msg = Message {
                    len: 0,
                    kind: 0,
                    body: Body::Jobs(vec![Job {
                        from: 10,
                        to: 20,
                        header: vec![3, 1, 1],
                        txs: vec![1, 1, 1],
                        target: vec![1, 1, 1],
                    }]),
                };

                println!("{:?}", msg);
                let server_message0_0 = bincode::encode_to_vec(msg, option).unwrap();
                println!("{:?}", server_message0_0);
                // [0, 0, 0, 38,
                // 0,
                // 0, 0, 0, 1,
                // 0, 0, 0, 10,
                // 0, 0, 0, 20,
                // 0, 0, 0, 3, 3, 1, 1,
                // 0, 0, 0, 3, 1, 1, 1,
                // 0, 0, 0, 3, 1, 1, 1]
                //4 + 1 + 3*4 + 7*3 = 5+ 12 + 21 = 38
            }
            let server_message0_0 = bincode::encode_to_vec(val.0.clone(), option);
            assert_eq!(
                server_message0_0.expect("server_message0_0 == decoded"),
                decoded
            );
        }
    }
}
