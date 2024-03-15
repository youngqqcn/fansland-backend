#!/usr/bin/env python

#author: yqq
#date: 2024-03-15
#description: telegram消息机器人

# https://github.com/python-telegram-bot/python-telegram-bot/blob/master/examples/echobot.py

import datetime
import logging
import os
import sys
import traceback
from dotenv import load_dotenv
import redis
from telegram import ForceReply, Update
from telegram.ext import Application,  ContextTypes, MessageHandler, filters

rds_conn = redis.Redis(host='localhost', port=6379, db=0, password='gooDluck4u')

async def message_handler(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    try:
        chat_id = context._chat_id
        user_id = context._user_id
        logging.info("chai_id: {}".format( chat_id))

        """Echo the user message."""
        msg_text = update.message.text
        if "#fansland" not in msg_text.lower():
            logging.info("消息格式不符")
            return

        logging.info("消息格式合法, 消息内容: {}".format(msg_text[: min(len(msg_text), 20)]))

        # 需求点: 记录用户发消息的次数
        logging.debug(f'合格的新消息, 消息发送者的id: {user_id}')
        # 使用 incr 增加 redis的计数器:
        # key的格式   gm:channel:渠道:日期:账户id
        #       渠道(discord、telegram)
        #       日期(2024-03-13)
        #       账户ID （1111111111111）
        # value: 发送消息的次数
        date = datetime.datetime.now().strftime("%Y-%m-%d")
        fmt_key = "gm:channel:{}:{}:{}".format("telegram", date, user_id)
        ret = rds_conn.incr(fmt_key)
        logging.info(f"ret:{ret}")

        # 暂不设置过期时间
        # if -1 == self.rds.ttl(fmt_key):
        #     ret = self.rds.expire(fmt_key, 24*60*60 + 1)
        #     logging.info(f"ret:{ret}")
    except Exception as e:
        traceback.print_exc(e)
        logging.exception(e)
    pass



def main() -> None:
    load_dotenv()
    chat_id = os.getenv( str(sys.argv[1]).strip() )
    token = os.getenv(str(sys.argv[2]).strip())


    logging.basicConfig(level=logging.INFO,
                    format='%(asctime)s.%(msecs)03d %(filename)s[line:%(lineno)d] %(levelname)s %(message)s',
                    datefmt='%Y-%m-%d %H:%M:%S')
    application = Application.builder().token(token).build()

    # 仅监听 指定群组的  文本类型的 消息
    application.add_handler(MessageHandler(filters.Chat(int(chat_id)) & filters.ChatType.GROUPS & filters.TEXT & ~filters.COMMAND, message_handler))


    application.run_polling(allowed_updates=Update.MESSAGE)


if __name__ == "__main__":
    main()