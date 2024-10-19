#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_primitives::{keccak256, Address, Bytes, FixedBytes, B256, U256};
use alloy_sol_types::{sol, SolValue};
use serde_json::Value;
use std::str::FromStr;

sol! {
  #[allow(missing_docs)]
  #[derive(Debug)]
  struct CompleteClaimData {
      bytes32 identifier;
      address owner;
      uint32 timestampS;
      uint32 epoch;
  }

  #[allow(missing_docs)]
  #[derive(Debug)]
  struct SignedClaim {
      CompleteClaimData claim;
      bytes[] signatures;
  }

  #[allow(missing_docs)]
  #[derive(Debug)]
  struct PublicValuesStruct {
    uint256 amount;
    bytes32 hashedClaimInfo;
    SignedClaim signedClaim;
  }
}

pub fn main() {
    let n = sp1_zkvm::io::read::<u32>();

    let json_data = r#"
    {
        "claimInfo": {
            "context": "{\"extractedParameters\":{\"text\":\"\\r\\nContent-Type: application/json; charset=utf-8\\r\\nTransfer-Encoding: chunked\\r\\nConnection: close\\r\\nx-frame-options: SAMEORIGIN\\r\\nx-xss-protection: 0\\r\\nx-content-type-options: nosniff\\r\\nx-download-options: noopen\\r\\nx-permitted-cross-domain-policies: none\\r\\nreferrer-policy: strict-origin-when-cross-origin\\r\\nCache-Control: max-age=30, public, must-revalidate, s-maxage=60\\r\\naccess-control-allow-origin: *\\r\\naccess-control-allow-methods: POST, PUT, DELETE, GET, OPTIONS\\r\\naccess-control-request-method: *\\r\\naccess-control-allow-headers: Origin, X-Requested-With, Content-Type, Accept, Authorization\\r\\naccess-control-expose-headers: link, per-page, total\\r\\nvary: Accept-Encoding, Origin\\r\\netag: W/\\\"8a72c9c13db83fc03cd53db9c10c32d4\\\"\\r\\nx-request-id: eafde21c-4142-4f9f-9155-bf222e70f969\\r\\nx-runtime: 0.002774\\r\\nalternate-protocol: 443:npn-spdy/2\\r\\nstrict-transport-security: max-age=15724800; includeSubdomains\\r\\nCF-Cache-Status: HIT\\r\\nAge: 135\\r\\nSet-Cookie: __cf_bm=xF6Q.U9S.1FBSjUStmRCBkVgqOQlbRy4yPeC3JaFhyg-1729186174-1.0.1.1-iC0xh7uviHV9kNES1Lbyox8VfvmdtZZnO.ZEEqKSZgRqZ2HxHQXvt5_Qvvp4xAzVRcSQ6a9_UO4HXfjeUqxnrB; path=/; expires=Thu, 17-Oct-24 17:59:34 GMT; domain=.api.coingecko.com; HttpOnly; Secure; SameSite=None\\r\\nServer: cloudflare\\r\\nCF-RAY: 8d4202f44ac13c22-BOM\\r\\nalt-svc: h3=\\\":443\\\"; ma=86400\\r\\n\\r\\n1c\\r\\n{\\\"ethereum\\\":{\\\"usd\\\":2605.74}}\\r\\n0\\r\\n\\r\\n\"},\"providerHash\":\"0xfb02c3a36651df46f97f30e22c729dbd2ed5ab1f40b03a88f5e7bafe55c74bfc\"}",
            "parameters": "{\"body\":\"\",\"method\":\"GET\",\"responseMatches\":[{\"type\":\"regex\",\"value\":\"(?\u003Ctext\u003E.*)\"}],\"responseRedactions\":[],\"url\":\"https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd\"}",
            "provider": "http"
        },
        "signedClaim": {
            "claim": {
                "epoch": 1,
                "identifier": "0x948156756472d7952981dac2ed87d2a304d6cf85d5359eeffe69b246640fa8e7",
                "owner": "0xa8e0c4777b3844cc019dbb33c67c2b4781f1aacf",
                "timestampS": 1729186174
            },
            "signatures": [
                "0x92416e3a55e2cc8f8691dee2d88457fafe5be85c79260be12080371edb8f51643903668f6510fc16c9e673ab4e1af3dff1d22d25c052a08860d5d6657d0527eb1b"
            ]
        }
    }
    "#;

    // =============================================================
    //                      Hashing the Claim Info
    // =============================================================

    // Parse the JSON data and get the claimInfo
    let parsed_data: Value = serde_json::from_str(json_data).unwrap();
    let context = parsed_data["claimInfo"]["context"].clone();
    let provider = parsed_data["claimInfo"]["provider"].clone();
    let parameters = parsed_data["claimInfo"]["parameters"].clone();

    let mut encoded_claim_info: Vec<u8> = Vec::new();
    encoded_claim_info.extend_from_slice(provider.to_string().as_bytes());
    encoded_claim_info.extend_from_slice(b"\n");
    encoded_claim_info.extend_from_slice(parameters.to_string().as_bytes());
    encoded_claim_info.extend_from_slice(b"\n");
    encoded_claim_info.extend_from_slice(context.to_string().as_bytes());

    let hashed_claim_info: B256 = keccak256(encoded_claim_info);

    println!("hashed_claim_info: {:?}", hashed_claim_info);

    // =============================================================
    //                   Public Values Structs
    // =============================================================

    let epoch = parsed_data["signedClaim"]["claim"]["epoch"]
        .as_u64()
        .unwrap() as u32;

    let identifier = FixedBytes::from_str(
        parsed_data["signedClaim"]["claim"]["identifier"]
            .as_str()
            .unwrap(),
    )
    .unwrap();

    let owner = Address::from_str(
        parsed_data["signedClaim"]["claim"]["owner"]
            .as_str()
            .unwrap(),
    )
    .unwrap();

    let timestamp = parsed_data["signedClaim"]["claim"]["timestampS"]
        .as_u64()
        .unwrap() as u32;

    let signatures = parsed_data["signedClaim"]["signatures"]
        .as_array()
        .unwrap()
        .iter() // Changed from into_iter() to iter()
        .map(|v| v.as_str().unwrap().to_string().into())
        .collect::<Vec<Bytes>>();

    // encode ABI
    let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct {
        amount: U256::from(n),
        hashedClaimInfo: hashed_claim_info,
        signedClaim: SignedClaim {
            claim: CompleteClaimData {
                epoch, // Convert Value to u32
                identifier,
                owner,
                timestampS: timestamp,
            },
            signatures,
        },
    });

    // Commit to the public values of the program. The final proof will have a commitment to all the
    // bytes that were committed to.
    sp1_zkvm::io::commit_slice(&bytes);
}
