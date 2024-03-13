#coding:utf8
# author: yqq
# date: 2024-03-13
# description: dicord任务机器人

import datetime
import logging
from typing import Generator
import discord
from discord.ext import tasks
from dotenv import load_dotenv
import sys
import os
import redis

class DiscordBotClient(discord.Client):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.rds = redis.Redis(host='localhost', port=6379, db=0, password='gooDluck4u')

        # an attribute we can access from our task
        self.global_invites = {}

    async def setup_hook(self) -> None:
        # start the task to run in the background
        self.invite_link_uses_task.start()
        self.update_all_members_task.start()

    def __print_invites(self, invts):
        for invt in invts:
            logging.debug(f'code:{invt.code},uses:{invt.uses},inviter_id:{invt.inviter.id},inviter:{invt.inviter},{invt.inviter.mention}')

    async def on_ready(self):
        logging.info(f'Logged in as {self.user} (ID: {self.user.id})')
        logging.info('------')
        for guild in self.guilds:
            invts = await guild.invites()
            self.global_invites[guild.id] = invts
            self.__print_invites(invts=invts)


    @tasks.loop(seconds=5)
    async def invite_link_uses_task(self):
        logging.info("=====更新邀请次数定时任务启动=======")
        for guild in self.guilds:
            invites = await guild.invites()
            # self.global_invites[guild.id] = invts
            self.__print_invites(invts=invites)
            for inv in invites:
                if inv.uses > 0:
                    logging.debug(f'{inv} uses is {inv.uses}')
                    # TODO: 插入redis / 数据库
        logging.info("=====更新邀请次数定时任务结束=======")
        pass

    @tasks.loop(seconds=5)
    async def update_all_members_task(self):
        logging.info("=====更新群成员定时任务启动=======")
        i = 0
        for member in super().get_all_members():
            logging.debug(f'成员{i}: {member}')
            i += 1
            # 向redis集合添加成员
            self.rds.sadd("discordmembers", str(member.id))
            pass
        logging.info("=====更新群成员定时任务结束=======")

    @update_all_members_task.before_loop
    async def before_task(self):
        await self.wait_until_ready()

    @invite_link_uses_task.before_loop
    async def before_invite_link_uses_task(self):
        await self.wait_until_ready()

    # async def on_member_join(self, member):
    #     """用户加入事件 """
    #     guild = member.guild
    #     if guild.system_channel is not None:
    #         to_send = f'Welcome {member.mention} to {guild.name}!'
    #         await guild.system_channel.send(to_send)

    async def on_message(self, message):
        """监听消息"""

        if message.author.id == self.user.id:
            logging.debug('机器人自己的消息')
            return

        # 需求点: 记录用户发消息的次数
        logging.debug(f'新消息, 消息发送者的id: {message.author.id}')
        # 使用 incr 增加 redis的计数器:
        # key的格式   gm:channel:渠道:日期:账户id
        #       渠道(discord、telegram)
        #       日期(2024-03-13)
        #       账户ID （1111111111111）
        # value: 发送消息的次数
        date = datetime.datetime.now().strftime("%Y-%m-%d")
        fmt_key = "gm:channel:{}:{}:{}".format("discord", date, message.author.id)
        ret = self.rds.incr(fmt_key)
        logging.info(f"ret:{ret}")

        # 暂不设置过期时间
        # if -1 == self.rds.ttl(fmt_key):
        #     ret = self.rds.expire(fmt_key, 24*60*60 + 1)
        #     logging.info(f"ret:{ret}")
        pass


def main():
    load_dotenv()
    logging.basicConfig(level=logging.DEBUG,
                    format='%(asctime)s.%(msecs)03d %(filename)s[line:%(lineno)d] %(levelname)s %(message)s',
                    datefmt='%Y-%m-%d %H:%M:%S')

    intents = discord.Intents.default()
    intents.members = True
    client = DiscordBotClient(intents=intents)

    # 从命令行获取参数名
    discord_bot_token = os.getenv( str(sys.argv[1]).strip() )
    client.run(discord_bot_token)
    pass

if __name__ == '__main__':
    main()
    pass

