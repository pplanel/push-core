use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use redis::{FromRedisValue, ToRedisArgs};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    game_id: usize,
    event_type: EventType,
    timestamp: u64,
}

impl FromRedisValue for Message {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match v {
            redis::Value::Data(data) => Ok(serde_json::from_slice(data).unwrap()),
            _ => todo!(),
        }
    }
}

impl ToRedisArgs for Message {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        out.write_arg(serde_json::to_vec(&self).unwrap().as_slice());
    }
}

impl Message {
    pub fn fake() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            game_id: rng.gen(),
            event_type: rand::random(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EventType {
    Start,      // (Início):
    End,        // (Fim):
    VARReview,  // VAR (Video Assistant Referee)
    Assist,     // (Assistência):
    Block,      // (Bloqueio):
    Card,       // (Cartão):
    Corner,     // (Escanteio):
    Freekick,   // (Falta):
    Goal,       // (Gol):
    Penalty,    // (Pênalti):
    PenaltyWon, // (Pênalti Ganho):
    Save,       // (Defesa):
    Shot,       // (Chute):
    SubOff,     // (Substituição - Saída):
    SubOn,      // (Substituição - Entrada):
    Throwin,    // (Arremesso Lateral):
}

impl Distribution<EventType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EventType {
        match rng.gen_range(0..=15) {
            0 => EventType::Start,
            1 => EventType::End,
            2 => EventType::VARReview,
            3 => EventType::Assist,
            4 => EventType::Block,
            5 => EventType::Card,
            6 => EventType::Corner,
            7 => EventType::Freekick,
            8 => EventType::Goal,
            9 => EventType::Penalty,
            10 => EventType::PenaltyWon,
            11 => EventType::Save,
            12 => EventType::Shot,
            13 => EventType::SubOff,
            14 => EventType::SubOn,
            15 => EventType::Throwin,
            _ => EventType::End,
        }
    }
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Start => write!(f, "Refere-se ao começo do jogo, quando a bola é inicialmente chutada pelo time que tem a posse no pontapé de saída."),
            EventType::End => write!(f, "Indica o término do jogo, quando o árbitro encerra a partida após o tempo regulamentar e, se necessário, acréscimos."),
            EventType::VARReview => write!(f, "O VAR (Video Assistant Referee) é um sistema de vídeo usado para revisar decisões tomadas pelo árbitro principal. O \"VAR Review\" ocorre quando uma decisão é contestada e a equipe de arbitragem decide revisar o incidente com a ajuda das imagens de vídeo."),
            EventType::Assist => write!(f, "Refere-se ao passe dado por um jogador que resulta em um gol marcado por outro jogador de sua equipe."),
            EventType::Block => write!(f, "Ocorre quando um jogador impede um chute ou passe do adversário ao posicionar seu corpo para interceptar a bola."),
            EventType::Card => write!(f, "Cartões amarelos e vermelhos são usados pelos árbitros para disciplinar os jogadores. Um cartão amarelo é um aviso e um cartão vermelho resulta na expulsão do jogador."),
            EventType::Corner => write!(f, "Concede-se um escanteio quando a bola atravessa a linha de fundo após ser tocada por um jogador da equipe defensora."),
            EventType::Freekick => write!(f, "Concede-se um tiro livre quando uma falta é cometida por um jogador, e a equipe prejudicada recebe a oportunidade de chutar diretamente ao gol ou avançar com a bola sem interferência da equipe adversária."),
            EventType::Goal => write!(f, "O objetivo principal do jogo, quando a bola atravessa completamente a linha do gol da equipe adversária."),
            EventType::Penalty => write!(f, "Concede-se um pênalti quando uma falta é cometida dentro da área de penalidade da equipe defensora. O jogador prejudicado recebe a oportunidade de chutar diretamente ao gol a partir da marca do pênalti."),
            EventType::PenaltyWon => write!(f, "Indica quando uma equipe recebe um pênalti devido a uma falta cometida pelo time adversário dentro da área de penalidade."),
            EventType::Save => write!(f, "Refere-se à intervenção feita pelo goleiro para impedir que a bola entre no gol."),
            EventType::Shot => write!(f, "Um chute é uma tentativa de um jogador de enviar a bola em direção ao gol adversário na esperança de marcar um gol."),
            EventType::SubOff => write!(f, "Indica quando um jogador é substituído e deixa o campo de jogo."),
            EventType::SubOn => write!(f, "Indica quando um jogador substituto entra no campo de jogo para substituir outro jogador que saiu."),
            EventType::Throwin => write!(f, "Concede-se um arremesso lateral quando a bola atravessa a linha lateral após ser tocada por um jogador e é reiniciada com um arremesso lateral."),
        }
    }
}
