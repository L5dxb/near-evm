use std::sync::Arc;

use futures::{future::LocalBoxFuture, FutureExt};

use near_crypto::{PublicKey, Signer};
use near_primitives::account::AccessKey;
use near_primitives::hash::CryptoHash;
use near_primitives::receipt::Receipt;
use near_primitives::transaction::{
    Action, AddKeyAction, CreateAccountAction, DeleteAccountAction, DeleteKeyAction,
    DeployContractAction, ExecutionOutcome, FunctionCallAction, SignedTransaction, StakeAction,
    TransferAction,
};
use near_primitives::types::{AccountId, Balance, BlockHeight, Gas, MerkleHash};
use near_primitives::views::{AccessKeyView, AccountView, BlockView, ViewStateResult};
use near_primitives::views::{ExecutionOutcomeView, FinalExecutionOutcomeView};

pub mod rpc_user;

// const POISONED_LOCK_ERR: &str = "The lock was poisoned.";

pub trait User {
    fn view_account(&self, account_id: &AccountId) -> Result<AccountView, String>;

    fn view_balance(&self, account_id: &AccountId) -> Result<Balance, String> {
        Ok(self.view_account(account_id)?.amount)
    }

    fn view_state(&self, account_id: &AccountId, prefix: &[u8]) -> Result<ViewStateResult, String>;

    fn add_transaction(&self, signed_transaction: SignedTransaction) -> Result<(), String>;

    fn commit_transaction(
        &self,
        signed_transaction: SignedTransaction,
    ) -> Result<FinalExecutionOutcomeView, String>;

    fn add_receipt(&self, receipt: Receipt) -> Result<(), String>;

    fn get_access_key_nonce_for_signer(&self, account_id: &AccountId) -> Result<u64, String> {
        self.get_access_key(account_id, &self.signer().public_key())
            .map(|access_key| access_key.nonce)
    }

    fn get_best_height(&self) -> Result<BlockHeight, String>;

    fn get_best_block_hash(&self) -> Result<CryptoHash, String>;

    fn get_block(&self, height: BlockHeight) -> Option<BlockView>;

    fn get_transaction_result(&self, hash: &CryptoHash) -> ExecutionOutcomeView;

    fn get_transaction_final_result(&self, hash: &CryptoHash) -> FinalExecutionOutcomeView;

    fn get_state_root(&self) -> CryptoHash;

    fn get_access_key(
        &self,
        account_id: &AccountId,
        public_key: &PublicKey,
    ) -> Result<AccessKeyView, String>;

    fn signer(&self) -> Arc<dyn Signer>;

    fn set_signer(&mut self, signer: Arc<dyn Signer>);

    fn sign_and_commit_actions(
        &self,
        signer_id: AccountId,
        receiver_id: AccountId,
        actions: Vec<Action>,
    ) -> Result<FinalExecutionOutcomeView, String> {
        let block_hash = self.get_best_block_hash().unwrap_or(CryptoHash::default());
        let signed_transaction = SignedTransaction::from_actions(
            self.get_access_key_nonce_for_signer(&signer_id).unwrap_or_default() + 1,
            signer_id,
            receiver_id,
            &*self.signer(),
            actions,
            block_hash,
        );
        self.commit_transaction(signed_transaction)
    }

    fn send_money(
        &self,
        signer_id: AccountId,
        receiver_id: AccountId,
        amount: Balance,
    ) -> Result<FinalExecutionOutcomeView, String> {
        self.sign_and_commit_actions(
            signer_id,
            receiver_id,
            vec![Action::Transfer(TransferAction { deposit: amount })],
        )
    }

    fn deploy_contract(
        &self,
        signer_id: AccountId,
        code: Vec<u8>,
    ) -> Result<FinalExecutionOutcomeView, String> {
        self.sign_and_commit_actions(
            signer_id.clone(),
            signer_id,
            vec![Action::DeployContract(DeployContractAction { code })],
        )
    }

    fn function_call(
        &self,
        signer_id: AccountId,
        contract_id: AccountId,
        method_name: &str,
        args: Vec<u8>,
        gas: Gas,
        deposit: Balance,
    ) -> Result<FinalExecutionOutcomeView, String> {
        self.sign_and_commit_actions(
            signer_id,
            contract_id,
            vec![Action::FunctionCall(FunctionCallAction {
                method_name: method_name.to_string(),
                args,
                gas,
                deposit,
            })],
        )
    }

    fn create_account(
        &self,
        signer_id: AccountId,
        new_account_id: AccountId,
        public_key: PublicKey,
        amount: Balance,
    ) -> Result<FinalExecutionOutcomeView, String> {
        self.sign_and_commit_actions(
            signer_id,
            new_account_id,
            vec![
                Action::CreateAccount(CreateAccountAction {}),
                Action::Transfer(TransferAction { deposit: amount }),
                Action::AddKey(AddKeyAction { public_key, access_key: AccessKey::full_access() }),
            ],
        )
    }

    fn add_key(
        &self,
        signer_id: AccountId,
        public_key: PublicKey,
        access_key: AccessKey,
    ) -> Result<FinalExecutionOutcomeView, String> {
        self.sign_and_commit_actions(
            signer_id.clone(),
            signer_id,
            vec![Action::AddKey(AddKeyAction { public_key, access_key })],
        )
    }

    fn delete_key(
        &self,
        signer_id: AccountId,
        public_key: PublicKey,
    ) -> Result<FinalExecutionOutcomeView, String> {
        self.sign_and_commit_actions(
            signer_id.clone(),
            signer_id,
            vec![Action::DeleteKey(DeleteKeyAction { public_key })],
        )
    }

    fn swap_key(
        &self,
        signer_id: AccountId,
        old_public_key: PublicKey,
        new_public_key: PublicKey,
        access_key: AccessKey,
    ) -> Result<FinalExecutionOutcomeView, String> {
        self.sign_and_commit_actions(
            signer_id.clone(),
            signer_id,
            vec![
                Action::DeleteKey(DeleteKeyAction { public_key: old_public_key }),
                Action::AddKey(AddKeyAction { public_key: new_public_key, access_key }),
            ],
        )
    }

    fn delete_account(
        &self,
        signer_id: AccountId,
        receiver_id: AccountId,
    ) -> Result<FinalExecutionOutcomeView, String> {
        self.sign_and_commit_actions(
            signer_id.clone(),
            receiver_id,
            vec![Action::DeleteAccount(DeleteAccountAction { beneficiary_id: signer_id })],
        )
    }

    fn stake(
        &self,
        signer_id: AccountId,
        public_key: PublicKey,
        stake: Balance,
    ) -> Result<FinalExecutionOutcomeView, String> {
        self.sign_and_commit_actions(
            signer_id.clone(),
            signer_id,
            vec![Action::Stake(StakeAction { stake, public_key })],
        )
    }
}

/// Same as `User` but provides async API that can be used inside tokio.
pub trait AsyncUser: Send + Sync {
    fn view_account(
        &self,
        account_id: &AccountId,
    ) -> LocalBoxFuture<'static, Result<AccountView, String>>;

    fn view_balance(
        &self,
        account_id: &AccountId,
    ) -> LocalBoxFuture<'static, Result<Balance, String>> {
        self.view_account(account_id).map(|res| res.map(|acc| acc.amount)).boxed_local()
    }

    fn view_state(
        &self,
        account_id: &AccountId,
    ) -> LocalBoxFuture<'static, Result<ViewStateResult, String>>;

    fn add_transaction(
        &self,
        transaction: SignedTransaction,
    ) -> LocalBoxFuture<'static, Result<(), String>>;

    fn add_receipt(&self, receipt: Receipt) -> LocalBoxFuture<'static, Result<(), String>>;

    fn get_account_nonce(
        &self,
        account_id: &AccountId,
    ) -> LocalBoxFuture<'static, Result<u64, String>>;

    fn get_best_height(&self) -> LocalBoxFuture<'static, Result<BlockHeight, String>>;

    fn get_transaction_result(
        &self,
        hash: &CryptoHash,
    ) -> LocalBoxFuture<'static, Result<ExecutionOutcome, String>>;

    fn get_transaction_final_result(
        &self,
        hash: &CryptoHash,
    ) -> LocalBoxFuture<'static, Result<FinalExecutionOutcomeView, String>>;

    fn get_state_root(&self) -> LocalBoxFuture<'static, Result<MerkleHash, String>>;

    fn get_access_key(
        &self,
        account_id: &AccountId,
        public_key: &PublicKey,
    ) -> LocalBoxFuture<'static, Result<Option<AccessKey>, String>>;
}