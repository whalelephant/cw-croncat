/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.19.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export interface InstantiateMsg {}
export type ExecuteMsg = string;
export type QueryMsg = {
  owner_of_nft: OwnerOfNft;
} | {
  addr_has_nft: {
    address: string;
    nft_address: string;
  };
};
export interface OwnerOfNft {
  address: string;
  nft_address: string;
  token_id: string;
}
export type Binary = string;
export interface QueryResponseForBinary {
  data: Binary;
  result: boolean;
}