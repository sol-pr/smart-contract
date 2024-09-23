import { serialize, deserialize, Schema } from "borsh";

export class User { 
  github_username: string = "";
  phantom_wallet: Uint8Array = new Uint8Array(32);
  totalearn: bigint = BigInt(0);
  submitted_at: bigint = BigInt(0);
  total_pr_count: bigint = BigInt(0);
  is_new_user: number = 0;


  constructor(fields: { github_username:string; phantom_wallet:Uint8Array; totalearn:bigint; submitted_at:bigint; total_pr_count:bigint; is_new_user:number;} | undefined = undefined) {
    if (fields) {
      this.github_username = fields.github_username;
      this.phantom_wallet = fields.phantom_wallet;
      this.totalearn = fields.totalearn;
      this.submitted_at = fields.submitted_at;
      this.total_pr_count = fields.total_pr_count;
      this.is_new_user = fields.is_new_user;
    }
  }
}

export const UserShema = new Map([
  [User, {
    kind: "struct",
    fields: [
      ["github_username", "String"],
      ["phantom_wallet", ["u8",32]],
      ["totalearn", "u64"],
      ["submitted_at", "u64"],
      ["total_pr_count", "u64"],
      ["is_new_user", "u8"],
    ]
  }]
]);


export class GithubRepo { 
  repo_url: string = "";
  repo_name: string = "";
  repo_description: string = "";
  total_pull_requests: bigint = BigInt(0);
  pull_request_limit: bigint = BigInt(0);
  reward_per_pull_request: bigint = BigInt(0);
  owner_wallet_address: Uint8Array = new Uint8Array(32);
  created_at: bigint = BigInt(0);


  constructor(fields: { repo_url: string; repo_name: string; repo_description: string; total_pull_requests:bigint; pull_request_limit:bigint; reward_per_pull_request:bigint; owner_wallet_address:Uint8Array ;created_at:bigint;} | undefined = undefined) {
    if (fields) {
      this.repo_url = fields.repo_url;
      this.repo_name = fields.repo_name;
      this.repo_description = fields.repo_description;
      this.total_pull_requests = fields.total_pull_requests;
      this.pull_request_limit = fields.pull_request_limit;
      this.reward_per_pull_request = fields.reward_per_pull_request;
      this.owner_wallet_address = fields.owner_wallet_address;
      this.created_at = fields.created_at;
    }
  }
}

export const GithubRepoShema = new Map([
  [GithubRepo, {
    kind: "struct",
    fields: [
      ["repo_url", "String"],
      ["repo_name", "String"],
      ["repo_description", "String"],
      ["total_pull_requests", "u64"],
      ["pull_request_limit", "u64"],
      ["reward_per_pull_request", "u64"],
      ["owner_wallet_address", ["u8",32]],
      ["created_at", "u64"],
    ]
  }]
]);



export class PrCount { 
  prcount: bigint = BigInt(0);

  constructor(fields: { prcount: bigint;} | undefined = undefined) {
    if (fields) {
      this.prcount = fields.prcount;

    }
  }
}

export const PrCountShema = new Map([
  [PrCount, {
    kind: "struct",
    fields: [
      ["prcount", "u64"],
    ]
  }]
]);