use scrypto::prelude::*;

#[derive(ScryptoSbor, Clone)]
pub struct Game {
    player1: Global<Account>,
    move1: Moves,
    player2: Option<Global<Account>>,
    move2: Option<Moves>,
    winner: Option<Global<Account>>,
    game_complete: bool,
    deadline: Instant,
    prize_collected: bool,
}

#[derive(ScryptoSbor, Clone, PartialEq)]
pub enum Moves {
    Rock,
    Paper,
    Rippy,
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
            player_move: Moves,
            room_code: String,
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

            if self.games.get(&room_code).is_none() {
                let game = Game {
                    player1: player.clone(),
                    move1: player_move,
                    player2: None,
                    move2: None,
                    winner: None,
                    game_complete: false,
                    deadline,
                    prize_collected: false,
                };

                self.games.insert(room_code, game.clone());
            } else {
                let game = self.games.get_mut(&room_code).unwrap().clone();

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

                    let player2 = player.clone();

                    let mut game = self.games.get_mut(&room_code).unwrap().clone();

                    game.player2 = Some(player2.clone());

                    game.move2 = Some(player_move.clone());

                    match (player1_move, player_move) {
                        // If both players make the same move, it's a tie
                        (move1, move2) if move1 == move2 => {
                            game.winner = None;
                        }

                        // Rock beats Rippy
                        (Moves::Rock, Moves::Rippy) => {
                            game.winner = Some(game.player1.clone());
                        }

                        // Rippy beats Paper
                        (Moves::Rippy, Moves::Paper) => {
                            game.winner = Some(game.player1.clone());
                        }

                        // Paper beats Rock
                        (Moves::Paper, Moves::Rock) => {
                            game.winner = Some(game.player1.clone());
                        }

                        // Otherwise, player 2 wins
                        _ => {
                            game.winner = Some(player2);
                        }
                    }

                    game.game_complete = true;

                    let game = game.clone();

                    self.games.insert(room_code, game.clone());
                }
            }
        }

        pub fn claim(&mut self, player: Global<Account>, room_code: String) -> Option<Bucket> {
            {
                let owner_role = player.get_owner_role();
                Runtime::assert_access_rule(owner_role.rule);
            }

            let game = self.games.get(&room_code).unwrap().clone();

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
                // draw wager logic also handles if the game didn't happen in time.

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
