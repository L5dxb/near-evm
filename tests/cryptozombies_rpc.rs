use std::sync::Arc;

use ethabi::{Address, Uint};
use ethabi_contract::use_contract;

use near_crypto::{InMemorySigner, KeyType};
use near_evm::{sender_name_to_eth_address};
use near_primitives::views::FinalExecutionStatus;
use near_primitives::serialize::from_base64;

mod user;
use user::User;
use user::rpc_user::RpcUser;

use_contract!(cryptozombies, "src/tests/zombieAttack.abi");

const CONTRACT_NAME: &str = "near_evm";

fn deploy_evm(client: &RpcUser, account_signer: &InMemorySigner) {
    println!("Deploying evm contract");
    let contract = include_bytes!("../res/near_evm.wasm").to_vec();
    let tx_result = client.deploy_contract(
        account_signer.account_id.clone(),
        contract);
    println!("Deploy evm contract: {:?}", tx_result);
}

fn deploy_cryptozombies(client: &RpcUser, account_signer: &InMemorySigner) {
    let zombie_code = include_bytes!("../src/tests/zombieAttack.bin").to_vec();
    let input = format!("{{\"contract_address\":\"cryptozombies\",\"bytecode\":\"{}\"}}", String::from_utf8(zombie_code).unwrap());
    let tx_result = client.function_call(
        account_signer.account_id.clone(),
        CONTRACT_NAME.to_string(),
        "deploy_code",
        input.into_bytes(),
        1_000_000_000,
        0);
    println!("deploy_code(cryptozombies): {:?}", tx_result);
}

fn create_random_zombie(client: &RpcUser, account_signer: &InMemorySigner, name: &str) {
    let (input, _decoder) = cryptozombies::functions::create_random_zombie::call(name.to_string());
    let input = format!("{{\"contract_address\":\"cryptozombies\",\"encoded_input\":\"{}\"}}", hex::encode(input));
    let tx_result = client.function_call(
        account_signer.account_id.clone(),
        CONTRACT_NAME.to_string(),
        "run_command",
        input.into_bytes(),
        1_000_000_000,
        0);
    println!("run_command(createRandomZombie): {:?}", tx_result);
}

fn get_zombies_by_owner(
    client: &RpcUser,
    account_signer: &InMemorySigner,
    owner: Address,
) -> Vec<Uint> {
    let (input, _decoder) = cryptozombies::functions::get_zombies_by_owner::call(owner);
    let input = format!("{{\"contract_address\":\"cryptozombies\",\"encoded_input\":\"{}\"}}", hex::encode(input));
    let tx_result = client.function_call(
        account_signer.account_id.clone(),
        CONTRACT_NAME.to_string(),
        "run_command",
        input.into_bytes(),
        1_000_000_000, 0);
    println!("run_command(getZombiesByOwner): {:?}", tx_result);
    if let FinalExecutionStatus::SuccessValue(ref base64) = tx_result.as_ref().unwrap().status {
        let bytes = from_base64(base64).unwrap();
        assert!(bytes.len() >= 2);
        let bytes = hex::decode(&bytes[1..bytes.len() - 1]).unwrap();
        cryptozombies::functions::get_zombies_by_owner::decode_output(&bytes).unwrap()
    } else {
        panic!(tx_result)
    }
}

fn run_test<T>(test: T) -> ()
    where T: FnOnce(&mut RpcUser, &mut RpcUser, &InMemorySigner) -> ()
{
    let addr = "localhost:3030";
    let signer = InMemorySigner::from_seed(
        CONTRACT_NAME,
        KeyType::ED25519,
        CONTRACT_NAME);
    let signer = Arc::new(signer);
    let mut user = RpcUser::new(addr, signer.account_id.clone(), signer.clone());

    let dev_signer = InMemorySigner::from_seed(
        "test.near",
        KeyType::ED25519,
        "alice.near");
    let devnet_account_id = dev_signer.account_id.clone();
    let mut dev_user = RpcUser::new(addr, signer.account_id.clone(), signer.clone());

    println!("\n");
    // let stat = user.get_status().unwrap();
    // println!("{:?}", stat);
    // println!("{:?}", user.get_block(stat.sync_info.latest_block_height));
    println!("\n");
    match user.get_best_height() {
        Ok(v) => println!("Ok {:?}", v),
        Err(v) => println!("Err {:?}", v)
    }
    println!("{:?}", user.get_best_height());
    // println!("{:?}", dev_user.get_best_height());
    // println!("\n");
    // println!("{:?}", user.get_best_block_hash());
    // println!("{:?}", dev_user.get_best_block_hash());
    // println!("\n");

    let tx_result = dev_user.create_account(
        devnet_account_id.clone(),
        signer.account_id.clone(),
        signer.public_key.clone(),
        10_000_000_000
    );

    let result = test(&mut user, &mut dev_user, &signer);
}

#[test]
fn test_rpc() {
    run_test(|user, dev_user, _| {
        println!("{:?}", user.get_status());
        println!("{:?}", dev_user.get_status());
    });
}
//
// #[test]
// fn test_zombie() {
//     let addr = "localhost:3030";
//     let signer = InMemorySigner::from_seed(CONTRACT_NAME, KeyType::ED25519, CONTRACT_NAME);
//     let signer = Arc::new(signer);
//     let mut user = RpcUser::new(addr, signer.account_id.clone(), signer.clone());
//     create_account(&mut user, &signer);
//     deploy_evm(&user, &signer);
//     deploy_cryptozombies(&user, &signer);
//     create_random_zombie(&user, &signer, "zomb1");
//     let zombies = get_zombies_by_owner(
//         &user,
//         &signer,
//         sender_name_to_eth_address(&signer.account_id),
//     );
//     assert_eq!(zombies, vec![Uint::from(0)]);
// }
