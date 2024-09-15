use scrypto::prelude::*;

#[derive(ScryptoSbor, Clone)]
pub struct Game {
    player1: Global<Account>,
    move1: String,
    player2: Option<Global<Account>>,
    move2: String,
    winner: Option<Global<Account>>,
    game_complete: bool,
    deadline: Instant,
    prize_collected: bool,
}

#[blueprint]
mod roshambo {
    struct Roshambo {
        games: KeyValueStore<String, Game>,
        prize_winnings: Vault,
    }

    impl Roshambo {
        pub fn instantiate_roshambo(clams: ResourceAddress) -> Global<Roshambo> {
            Self {
                games: KeyValueStore::new(),
                prize_winnings: Vault::new(clams),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn play_game(
            &mut self,
            player: Global<Account>,
            move1: String,
            port: String,
            clam: Bucket,
        ) {
            {
                let owner_role = player.get_owner_role();
                Runtime::assert_access_rule(owner_role.rule);
            }

            assert!(
                clam.resource_address() == self.prize_winnings.resource_address(),
                "Sorry, we only play for clams round here."
            );

            assert!(clam.amount() == dec!(1), "Well hello there high roller, that wager is too rich for our blood. Jetty's clams aren't to be thrown away like that.");

            self.prize_winnings.put(clam);

            let current_time = Clock::current_time_rounded_to_seconds();

            let deadline = current_time.add_seconds(8).unwrap();

            if self.games.get(&port).is_none() {
                let game = Game {
                    player1: player.clone(),
                    move1,
                    player2: None,
                    move2: "".to_string(),
                    winner: None,
                    game_complete: false,
                    deadline,
                    prize_collected: false,
                };

                self.games.insert(port, game.clone());
            } else {
                let game = self.games.get_mut(&port).unwrap().clone();

                let game_time = game.deadline;

                let game_winner = game.winner;

                if game_winner.is_some() {
                    panic!(
                        "Game is already over! The winner was: {:?}",
                        game_winner.unwrap()
                    );
                }

                if current_time.compare(game_time, TimeComparisonOperator::Gt) {
                    panic!("Too slow! You didn't throw down your move quick enough. ");
                } else {
                    // possible players moves - 'Rock', 'Paper', 'Rippy'
                    // Rock beats Rippy, Rippy beats Paper, Paper beats Rock

                    let player1_move = game.move1;

                    let player2_move = move1;

                    let player2 = player.clone();

                    let mut game = self.games.get_mut(&port).unwrap().clone();

                    game.player2 = Some(player2);

                    game.move2 = player2_move.clone();

                    if player1_move == player2_move {
                        game.winner = None;
                    } else if player1_move == "Rock" && player2_move == "Rippy" {
                        game.winner = Some(game.player1.clone());
                    } else if player1_move == "Rippy" && player2_move == "Paper" {
                        game.winner = Some(game.player1.clone());
                    } else if player1_move == "Paper" && player2_move == "Rock" {
                        game.winner = Some(game.player1.clone());
                    } else {
                        game.winner = Some(game.player2.clone().unwrap());
                    }

                    game.game_complete = true;

                    let game = game.clone();

                    self.games.insert(port, game.clone());
                }
            }
        }

        pub fn claim_winnings(&mut self, player: Global<Account>, port: String) -> Option<Bucket> {
            {
                let owner_role = player.get_owner_role();
                Runtime::assert_access_rule(owner_role.rule);
            }

            let game = self.games.get(&port).unwrap().clone();

            if game.winner.is_some() {
                let winner = game.winner.unwrap();

                assert!(game.prize_collected, "Already claimed the winnings.");
                assert!(game.game_complete, "Game is still in progress.");

                if winner == player {
                    let prize = self.prize_winnings.take(dec!(2));

                    return Some(prize);
                } else {
                    panic!("You didn't win, so you can't collect the prize.");
                }
            } else {
                assert!(game.game_complete, "Game is still in progress.");

                if game.player1 == player {
                    let draw_wager = self.prize_winnings.take(dec!(1));

                    return Some(draw_wager);
                } else {
                    let player2 = game.player2.unwrap();

                    if player2 == player {
                        let draw_wager = self.prize_winnings.take(dec!(1));

                        return Some(draw_wager);
                    };
                }

                None
            }
        }
    }
}
