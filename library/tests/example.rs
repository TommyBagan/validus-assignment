use chrono::{Duration, TimeDelta, Utc};
use iso_currency::Currency;
use library::{history::{get_historical_record, total_historical_record_count}, state::{Approved, Draft, NeedsReapproval, PendingApproval, TradeAction}, trade::{Counterparty, Direction, MutTradeDetails, Style, TradeDetails}, users::{Approver, Requester, User}};

#[test]
/// This test works an example for the various interacts with the API.
fn example_updates_and_history() {
    // This demonstrates we should have no records now.
    assert_eq!(total_historical_record_count(), 0);
    
    // First Bob will sign in.
    let bob: User<Requester> = User::<Requester>::sign_in("Bob");

    // Bob will initially create a draft trade.
    let trade: TradeDetails<Draft> = TradeDetails::<Draft>::new(
        &bob, 
        Counterparty("Maggie".to_string()), 
        Direction::BUY, 
        Style("Forward Contract Currency Exchange.".to_string()),
        iso_currency::Currency::USD, 
        1, 
        vec![Currency::USD, Currency::GBP], 
        Utc::now() + Duration::from(TimeDelta::days(365)), 
        Utc::now() + Duration::from(TimeDelta::days(366))
    ).unwrap();

    // Bob submits this draft, for Ellie to view.
    let trade: TradeDetails<PendingApproval> = trade.submit(&bob).unwrap();
    assert_eq!(total_historical_record_count(), 1);

    // Ellie now signs in.
    let ellie: User<Approver> = User::<Approver>::sign_in("Ellie");
    
    // Ellie has a look at bobs draft, notices he's trading very little.
    assert_eq!(trade.amount(), 1);

    // Probably a typo - typical Bob. Ellie updates his issue instead of approving.
    let mut new_trade = trade.grab_mut_details();
    new_trade.notional_amount = 1000;
    let trade: TradeDetails<NeedsReapproval> = trade.update(&ellie, new_trade).unwrap();

    // Bob gets the latest change history to find out whether his trade was approved.
    assert_eq!(total_historical_record_count(), 2);
    let record = get_historical_record(total_historical_record_count() - 1).unwrap();
    
    // Bob notices his trade has been updated!
    assert_eq!(record.action(), &TradeAction::Update);
    assert_eq!(record.changes().unwrap().changed_amount().unwrap(), (1, 1000));

    // Looks like he made a typo, and so he reapproves.
    let trade: TradeDetails<Approved> = trade.approve(&bob).unwrap();
    assert_eq!(total_historical_record_count(), 3);

    // Happy now, Ellie sends and eventually completes the trade.
    trade.send_to_execute(&ellie)
        .book(900, &ellie);
    assert_eq!(total_historical_record_count(), 5);
}