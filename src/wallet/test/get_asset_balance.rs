use super::*;

#[test]
fn success() {
    initialize();

    let (mut wallet, online) = get_funded_wallet!();

    // no assets
    let asset_list = wallet.list_assets().unwrap();
    assert_eq!(asset_list.len(), 0);

    // one issued asset
    let asset = wallet
        .issue_asset(
            online.clone(),
            TICKER.to_string(),
            NAME.to_string(),
            PRECISION,
            AMOUNT,
        )
        .unwrap();
    let asset_balance = wallet.get_asset_balance(asset.asset_id).unwrap();
    assert_eq!(
        asset_balance,
        Balance {
            settled: AMOUNT,
            future: AMOUNT
        }
    );
}

#[test]
fn transfer_balances() {
    initialize();

    let amount_1: u64 = 66;
    let amount_2: u64 = 33;

    let (mut wallet_send, online_send) = get_funded_wallet!();
    let (mut wallet_recv, online_recv) = get_funded_wallet!();

    // issue
    let asset = wallet_send
        .issue_asset(
            online_send.clone(),
            TICKER.to_string(),
            NAME.to_string(),
            PRECISION,
            AMOUNT,
        )
        .unwrap();

    // balances after issuance
    let asset_balance_send = wallet_send
        .get_asset_balance(asset.asset_id.clone())
        .unwrap();
    assert_eq!(
        asset_balance_send,
        Balance {
            settled: AMOUNT,
            future: AMOUNT
        }
    );
    // receiver side after issuance (no asset yet)
    let asset_balance_recv = wallet_recv
        .get_asset_balance(asset.asset_id.clone())
        .unwrap();
    assert_eq!(
        asset_balance_recv,
        Balance {
            settled: 0,
            future: 0
        }
    );

    //
    // 1st transfer
    //

    // send some assets
    let blind_data_1 = wallet_recv.blind(None, None).unwrap();
    wallet_send
        .send(
            online_send.clone(),
            asset.asset_id.clone(),
            blind_data_1.blinded_utxo,
            amount_1,
        )
        .unwrap();

    // sender balance with transfer WaitingCounterparty
    let transfers = wallet_send.list_transfers(asset.asset_id.clone()).unwrap();
    assert_eq!(transfers.len(), 2);
    assert_eq!(
        transfers.last().unwrap().status,
        TransferStatus::WaitingCounterparty
    );
    let asset_balance_send = wallet_send
        .get_asset_balance(asset.asset_id.clone())
        .unwrap();
    assert_eq!(
        asset_balance_send,
        Balance {
            settled: AMOUNT,
            future: AMOUNT - amount_1
        }
    );

    // take transfers from WaitingCounterparty to WaitingConfirmations
    wallet_recv.refresh(online_recv.clone(), None).unwrap();
    wallet_send
        .refresh(online_send.clone(), Some(asset.asset_id.clone()))
        .unwrap();

    // balances with transfer WaitingConfirmations
    let transfers = wallet_send.list_transfers(asset.asset_id.clone()).unwrap();
    assert_eq!(
        transfers.last().unwrap().status,
        TransferStatus::WaitingConfirmations
    );
    let asset_balance_send = wallet_send
        .get_asset_balance(asset.asset_id.clone())
        .unwrap();
    assert_eq!(
        asset_balance_send,
        Balance {
            settled: AMOUNT,
            future: AMOUNT - amount_1
        }
    );
    let asset_balance_recv = wallet_recv
        .get_asset_balance(asset.asset_id.clone())
        .unwrap();
    assert_eq!(
        asset_balance_recv,
        Balance {
            settled: 0,
            future: amount_1
        }
    );

    // take transfers from WaitingConfirmations to Settled
    mine();
    wallet_recv
        .refresh(online_recv.clone(), Some(asset.asset_id.clone()))
        .unwrap();
    wallet_send
        .refresh(online_send.clone(), Some(asset.asset_id.clone()))
        .unwrap();

    // balances with transfer Settled
    let transfers = wallet_send.list_transfers(asset.asset_id.clone()).unwrap();
    assert_eq!(transfers.last().unwrap().status, TransferStatus::Settled);
    let asset_balance_send = wallet_send
        .get_asset_balance(asset.asset_id.clone())
        .unwrap();
    assert_eq!(
        asset_balance_send,
        Balance {
            settled: AMOUNT - amount_1,
            future: AMOUNT - amount_1
        }
    );
    let asset_balance_recv = wallet_recv
        .get_asset_balance(asset.asset_id.clone())
        .unwrap();
    assert_eq!(
        asset_balance_recv,
        Balance {
            settled: amount_1,
            future: amount_1
        }
    );

    //
    // a 2nd transfer
    //

    // send some assets
    let blind_data_2 = wallet_recv.blind(None, None).unwrap();
    wallet_send
        .send(
            online_send.clone(),
            asset.asset_id.clone(),
            blind_data_2.blinded_utxo,
            amount_2,
        )
        .unwrap();

    // sender balance with transfer WaitingCounterparty
    let transfers = wallet_send.list_transfers(asset.asset_id.clone()).unwrap();
    assert_eq!(transfers.len(), 3);
    assert_eq!(
        transfers.last().unwrap().status,
        TransferStatus::WaitingCounterparty
    );
    let asset_balance_send = wallet_send
        .get_asset_balance(asset.asset_id.clone())
        .unwrap();
    assert_eq!(
        asset_balance_send,
        Balance {
            settled: AMOUNT - amount_1,
            future: AMOUNT - amount_1 - amount_2
        }
    );

    // take transfers from WaitingCounterparty to WaitingConfirmations
    wallet_recv.refresh(online_recv.clone(), None).unwrap();
    wallet_send
        .refresh(online_send.clone(), Some(asset.asset_id.clone()))
        .unwrap();

    // balances with transfer WaitingConfirmations
    let transfers = wallet_send.list_transfers(asset.asset_id.clone()).unwrap();
    assert_eq!(
        transfers.last().unwrap().status,
        TransferStatus::WaitingConfirmations
    );
    let asset_balance_send = wallet_send
        .get_asset_balance(asset.asset_id.clone())
        .unwrap();
    assert_eq!(
        asset_balance_send,
        Balance {
            settled: AMOUNT - amount_1,
            future: AMOUNT - amount_1 - amount_2
        }
    );
    let asset_balance_recv = wallet_recv
        .get_asset_balance(asset.asset_id.clone())
        .unwrap();
    assert_eq!(
        asset_balance_recv,
        Balance {
            settled: amount_1,
            future: amount_1 + amount_2
        }
    );

    // take transfers from WaitingConfirmations to Settled
    mine();
    wallet_recv
        .refresh(online_recv.clone(), Some(asset.asset_id.clone()))
        .unwrap();
    wallet_send
        .refresh(online_send.clone(), Some(asset.asset_id.clone()))
        .unwrap();

    // balances with transfer Settled
    let transfers = wallet_send.list_transfers(asset.asset_id.clone()).unwrap();
    assert_eq!(transfers.last().unwrap().status, TransferStatus::Settled);
    let asset_balance_send = wallet_send
        .get_asset_balance(asset.asset_id.clone())
        .unwrap();
    assert_eq!(
        asset_balance_send,
        Balance {
            settled: AMOUNT - amount_1 - amount_2,
            future: AMOUNT - amount_1 - amount_2
        }
    );
    let asset_balance_recv = wallet_recv
        .get_asset_balance(asset.asset_id.clone())
        .unwrap();
    assert_eq!(
        asset_balance_recv,
        Balance {
            settled: amount_1 + amount_2,
            future: amount_1 + amount_2
        }
    );
}