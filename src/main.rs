use std::collections::{HashMap, HashSet};

use position::{Dir, Neighbors, Pos, START};
use serenity::{
    async_trait,
    framework::{
        standard::{
            buckets::LimitedFor,
            help_commands,
            macros::{command, group, help, hook},
            Args, CommandGroup, CommandResult, DispatchError, HelpOptions,
        },
        StandardFramework,
    },
    futures::future::BoxFuture,
    http::Http,
    model::{
        channel::Message,
        id::{ChannelId, UserId},
        prelude::Ready,
    },
    prelude::*,
    FutureExt,
};
use A_star::A_star;

mod A_star;
mod position;

struct ChatsAndBoars;

impl TypeMapKey for ChatsAndBoars {
    type Value = HashMap<ChannelId, Pos>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected", ready.user.name)
    }
}

#[group]
#[commands(start, solution, refresh, maze, up, down, left, right)]
struct General;

#[help]
#[individual_command_tip = "Hello!\n\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn my_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners);
    Ok(())
}

#[hook]
async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    true
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returnder error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Message in not a command '{}'", msg.content);
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again {} seconds.", info.as_secs()),
                )
                .await;
        }
    }
}

fn _dispatch_error_no_macro<'fut>(
    ctx: &'fut mut Context,
    msg: &'fut Message,
    error: DispatchError,
) -> BoxFuture<'fut, ()> {
    async move {
        if let DispatchError::Ratelimited(info) = error {
            if info.is_first_try {
                let _ = msg.channel_id.say(
                    &ctx.http,
                    &format!("Try this again in {} seconds.", info.as_secs()),
                );
            }
        }
    }
    .boxed()
}

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    let token = "ODc2MjM2NjQzNjk3MzI4MTQw.YRhI9w.zw9z3OImj0t2vrfXkfg0mBiQmDI";

    let http = Http::new_with_token(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();

            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }

            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => {
            panic!("Could not access the bot id: {:?}", why);
        }
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix("~")
                .delimiters(vec![", ", ","])
                .owners(owners)
        })
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .bucket("emoji", |b| b.delay(5))
        .await
        .bucket("complicated", |b| {
            b.limit(2)
                .time_span(30)
                .delay(5)
                .limit_for(LimitedFor::Channel)
                .await_ratelimits(1)
                .delay_action(delay_action)
        })
        .await
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .unwrap();

    {
        let mut data = client.data.write().await;
        data.insert::<ChatsAndBoars>(HashMap::default());
    }

    if let Err(why) = client.start().await {
        println!("Client error {:?}", why);
    }
}

#[command]
async fn start(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let games = data.get_mut::<ChatsAndBoars>().unwrap();
    games.insert(msg.channel_id, START);
    msg.reply(ctx, format!("{}", START)).await?;

    Ok(())
}

#[command]
async fn solution(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.write().await;
    let games = data.get::<ChatsAndBoars>().unwrap();
    if let Some(pos) = games.get(&msg.channel_id) {
        let solution = A_star::solution(A_star(*pos));
        let text = solution
            .into_iter()
            .map(|x| format!("{}", x))
            .fold(String::new(), |a, b| a + b.as_str() + "\n");

        msg.reply(ctx, text).await?;
    }

    Ok(())
}

#[command]
async fn refresh(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let games = data.get_mut::<ChatsAndBoars>().unwrap();
    games.insert(msg.channel_id, START);
    msg.reply(ctx, format!("{}", START)).await?;

    Ok(())
}

#[command]
async fn maze(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let games = data.get_mut::<ChatsAndBoars>().unwrap();
    let new_pos = position::maze();
    games.insert(msg.channel_id, new_pos);

    msg.reply(ctx, format!("{}", new_pos)).await?;

    Ok(())
}

fn make_step(map: &mut HashMap<ChannelId, Pos>, id: ChannelId, dir: Dir) -> Option<Pos> {
    map.get_mut(&id).map(|pos| {
        *pos = pos.apply(dir);
        *pos
    })
}

#[command]
async fn up(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let games = data.get_mut::<ChatsAndBoars>().unwrap();

    if let Some(pos) = make_step(games, msg.channel_id, Dir::Up) {
        msg.reply(ctx, format!("{}", pos)).await?;
    }

    Ok(())
}

#[command]
async fn down(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let games = data.get_mut::<ChatsAndBoars>().unwrap();

    if let Some(pos) = make_step(games, msg.channel_id, Dir::Down) {
        msg.reply(ctx, format!("{}", pos)).await?;
    }

    Ok(())
}

#[command]
async fn left(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let games = data.get_mut::<ChatsAndBoars>().unwrap();

    if let Some(pos) = make_step(games, msg.channel_id, Dir::Left) {
        msg.reply(ctx, format!("{}", pos)).await?;
    }

    Ok(())
}

#[command]
async fn right(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let games = data.get_mut::<ChatsAndBoars>().unwrap();

    if let Some(pos) = make_step(games, msg.channel_id, Dir::Right) {
        msg.reply(ctx, format!("{}", pos)).await?;
    }

    Ok(())
}
