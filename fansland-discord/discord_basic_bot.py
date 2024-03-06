# This example requires the 'members' and 'message_content' privileged intents to function.

import discord
from discord.ext import commands
import random
import asyncio

description = '''An example bot to showcase the discord.ext.commands extension
module.

There are a number of utility commands being showcased here.'''

intents = discord.Intents.default()
intents.members = True
intents.message_content = True

bot = commands.Bot(command_prefix='?', description=description, intents=intents)

global_invites = {}  # 存储邀请者信息的字典，可以使用数据库替代

def print_invite(invt):
    print(f'code:{invt.code},uses:{invt.uses},inviter_id:{invt.inviter.id},inviter:{invt.inviter},{invt.inviter.mention}')

def print_invites(invts):
    for invt in invts:
        print_invite(invt)


@bot.event
async def on_ready():
    print(f'Logged in as {bot.user} (ID: {bot.user.id})')
    print('------')
    for guild in bot.guilds:
        invts = await guild.invites()
        global_invites[guild.id] = invts
        for ivt in invts:
            print_invite(ivt)
    # print(f"global_invites:{global_invites}")


@bot.command()
async def add(ctx, left: int, right: int):
    """Adds two numbers together."""
    await ctx.send(left + right)


@bot.command()
async def roll(ctx, dice: str):
    """Rolls a dice in NdN format."""
    try:
        rolls, limit = map(int, dice.split('d'))
    except Exception:
        await ctx.send('Format has to be in NdN!')
        return

    result = ', '.join(str(random.randint(1, limit)) for r in range(rolls))
    await ctx.send(result)


@bot.command(description='For when you wanna settle the score some other way')
async def choose(ctx, *choices: str):
    """Chooses between multiple choices."""
    await ctx.send(random.choice(choices))


@bot.command()
async def repeat(ctx, times: int, content='repeating...'):
    """Repeats a message multiple times."""
    for i in range(times):
        await ctx.send(content)


@bot.command()
async def joined(ctx, member: discord.Member):
    """Says when a member joined."""
    await ctx.send(f'{member.name} joined {discord.utils.format_dt(member.joined_at)}')


@bot.group()
async def cool(ctx):
    """Says if a user is cool.

    In reality this just checks if a subcommand is being invoked.
    """
    if ctx.invoked_subcommand is None:
        await ctx.send(f'No, {ctx.subcommand_passed} is not cool')


@cool.command(name='bot')
async def _bot(ctx):
    """Is the bot cool?"""
    await ctx.send('Yes, the bot is cool.')




def find_invite_by_code( inv_list, code):
    for inv in inv_list:
        if inv.code == code:
            return inv

@bot.event
async def on_member_join(member):
    invs_before = global_invites[member.guild.id]
    print(f"invs_before: {invs_before}")
    print('=====================')
    invs_after = await member.guild.invites()
    print(f"invs_after: {invs_after}")
    print('=====================')
    global_invites[member.guild.id] = invs_after
    for invite in invs_before:
        # 通过对比 uses 来记录邀请成功的人数
        # 参考： https://github.com/GregTCLTK/Discord-Invite-Tracker/blob/7b3f397e26d1953fe3609e5bd72dcfe7849b799f/invite_tracker.py#L58
        tmp_inv = find_invite_by_code(invs_after, invite.code)
        if invite.uses < tmp_inv.uses:
            print(f"Inviter: {invite.inviter.mention} (`{invite.inviter}` | `{str(invite.inviter.id)}`)\nCode: `{invite.code}`\nUses: ` {str(tmp_inv.uses)} `")
            pass



#TODO: on_invite_create

@bot.command()
async def invite(ctx):
    inviter = ctx.author
    print(inviter)
    print(inviter.id)
    invite = await ctx.channel.create_invite()
    invs_after = await ctx.author.guild.invites()
    global_invites[ctx.author.guild.id] = invs_after
    print_invites(invs_after)

    # global_invites[invite.url] = inviter.id  # 存储邀请者信息到字典中
    await ctx.send(f'Here is your invite link: {invite.url}')





bot.run('MTIxNDE0ODQxNDAyODEyMDExNQ.GjFbH4.SP1Czvehldc0W0BUPXfcZHYfQI9qDIgWxPex04')
