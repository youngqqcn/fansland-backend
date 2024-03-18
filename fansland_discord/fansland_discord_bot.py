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
import traceback

class DiscordBotClient(discord.Client):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.rds = redis.Redis(host='localhost', port=6379, db=0, password='gooDluck4u')

        # an attribute we can access from our task
        self.global_invites = {}

    async def setup_hook(self) -> None:
        # start the task to run in the background
        # self.invite_link_uses_task.start()
        self.update_all_members_task.start()
        pass

    def __print_invites(self, invts):
        for invt in invts:
            logging.debug(f'code:{invt.code},uses:{invt.uses},inviter_id:{invt.inviter.id},inviter:{invt.inviter},{invt.inviter.mention}')

    async def on_ready(self):
        try:
            logging.info(f'Logged in as {self.user} (ID: {self.user.id})')
            logging.info('------')
            for guild in self.guilds:
                invts = await guild.invites()
                self.global_invites[guild.id] = invts
                self.__print_invites(invts=invts)
        except Exception as e:
            logging.error(e)


    def __get_start_of_week(self):
        # 获取当前日期
        today = datetime.date.today()

        # 获取当前是第几周
        week_number = today.isocalendar()[1]

        # 获取当前星期几（0表示星期一，6表示星期日）
        weekday = today.weekday()

        # 获取本周的第一天日期
        start_of_week = today - datetime.timedelta(days=today.weekday())

        return start_of_week.strftime("%Y-%m-%d")


    # async def update_invite_count(self):
    #     logging.info("=====更新邀请次数定时任务启动=======")
    #     for guild in self.guilds:
    #         invites = await guild.invites()
    #         # self.global_invites[guild.id] = invts
    #         self.__print_invites(invts=invites)
    #         for inv in invites:
    #             if inv.uses > 0:
    #                 logging.debug(f'{inv} uses is {inv.uses}')
    #                 # 插入redis / 数据库
    #                 fmt_key = "gm:invitelinkuses:{}".format(str(inv.inviter.id))
    #                 self.rds.set(fmt_key, inv.uses)
    #     logging.info("=====更新邀请次数定时任务结束=======")
    #     pass

    # @invite_link_uses_task.before_loop
    # async def before_invite_link_uses_task(self):
        # await self.wait_until_ready()

    @tasks.loop(seconds=10)
    async def update_all_members_task(self):
        try:
            logging.info("=====更新群成员定时任务启动=======")
            for member in super().get_all_members():
                # logging.debug(f'成员{i}: {member}')
                # 向redis集合添加成员
                self.rds.sadd("gm:discord:members", str(member.id))
                pass
            logging.info("=====更新群成员定时任务结束=======")
        except Exception as e:
            logging.error(e)


    @update_all_members_task.before_loop
    async def before_task(self):
        await self.wait_until_ready()


    def __find_invite_by_code(self, inv_list, code):
        for inv in inv_list:
            if inv.code == code:
                return inv
        return None

    async def on_member_join(self, member: discord.Member):
        """用户加入事件 """
        try:
            logging.info(f"用户:{member.id} 加入")
            invs_before = self.global_invites[member.guild.id]
            logging.debug(f"invs_before: {len(invs_before)}")
            logging.debug('=====================')
            invs_after = await member.guild.invites()
            logging.debug(f"invs_after: {len(invs_after)}")
            logging.debug('=====================')
            self.global_invites[member.guild.id] = invs_after

            # 判断这个用户之前有没有加入过，
            records_key = "gm:discord:inviterecords"
            ret = self.rds.sismember(records_key, str(member.id))
            if str(ret) == '1':
                logging.info(f"此用户:{member.id} 以前加入过，不再计算邀请次数")
                return

            # 如果是新用户，则计算邀请次数
            for new_invite in invs_after:
                # 通过对比 uses 来记录邀请成功的人数
                # 参考： https://github.com/GregTCLTK/Discord-Invite-Tracker/blob/7b3f397e26d1953fe3609e5bd72dcfe7849b799f/invite_tracker.py#L58
                # 在最新的集合中找
                is_new_invite = False
                ret_invt = self.__find_invite_by_code(invs_before, new_invite.code)
                if ret_invt == None:
                    if new_invite.uses == 1:
                        # 第一个邀请
                        is_new_invite = True
                elif ret_invt.uses < new_invite.uses :
                    logging.info(f"Inviter: {ret_invt.inviter.mention} (`{ret_invt.inviter}` | `{str(ret_invt.inviter.id)}`)\nCode: `{ret_invt.code}`\nUses: ` {str(new_invite.uses)} `")
                    is_new_invite = True
                    pass
                if is_new_invite:
                    # 是新的邀请, 将被邀请的做个记录，防止重复退出有加入，重复刷
                    self.rds.sadd(records_key, str(member.id))
                    # 邀请计数新增1
                    self.rds.incr("gm:dicords:invitecounter:{}:{}".format(new_invite.inviter.id, self.__get_start_of_week()))

                    # 找到邀请人就结束
                    return

            logging.info("用户:{member.id}未找邀请人")
        except Exception as e:
            traceback.print_exc(e)
            logging.error(e)
        pass

    async def on_message(self, message: discord.Message):
        """监听消息"""

        try:
            if message.author.id == self.user.id:
                logging.debug('机器人自己的消息')
                return

            msg_text = message.content
            if "#fansland" not in msg_text.lower():
                logging.info("消息格式不符")
                return

            # 需求点: 记录用户发消息的次数
            logging.debug(f'合格的新消息, 消息发送者的id: {message.author.id}')
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
        except Exception as e:
            logging.error(e)
            pass
        pass


def main():
    load_dotenv()
    logging.basicConfig(level=logging.DEBUG,
                    format='%(asctime)s.%(msecs)03d %(filename)s[line:%(lineno)d] %(levelname)s %(message)s',
                    datefmt='%Y-%m-%d %H:%M:%S')

    intents = discord.Intents.all()
    intents.members = True
    intents.messages = True
    client = DiscordBotClient(intents=intents)

    # 从命令行获取参数名
    discord_bot_token = os.getenv( str(sys.argv[1]).strip() )
    client.run(discord_bot_token)
    pass

if __name__ == '__main__':
    main()
    pass

