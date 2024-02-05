import os
import boto3
from botocore.exceptions import ClientError
from email.mime.multipart import MIMEMultipart
from email.mime.text import MIMEText
from email.mime.image import MIMEImage
from email.mime.application import MIMEApplication
import os
from .template import get_mint_template
from .gen_qrcode import gen_qrcode_png_bytes


class SendEmailMsg(object):
    def __init__(self, to_email, qrcodes, address, chain, chainid, ticket_type, token_id):
        self.qrcodes=qrcodes
        self.email = to_email
        self.address = address
        self.chain= chain
        self.chainid = chainid
        self.ticket_type = ticket_type
        self.token_id = token_id
        pass

    def __repr__(self) :
        return 'qrcodes:{}, email:{},address:{},chain:{}'.format(
            self.qrcodes, self.email, self.address, self.chain)


class SendEmail:
    def __init__(self):
        AWS_REGION="ap-northeast-1"

        # Create a new SES resource and specify a region.
        self.client = boto3.client('ses',region_name=AWS_REGION,
            aws_access_key_id='AKIAWTXOSMHXSIT5FEFQ',
                aws_secret_access_key='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')
        pass

    def send_raw_email(self, send_msg):

        SENDER = "Fansland <no-reply@fansland.io>"

        # The subject line for the email.
        SUBJECT = "Fansland NFT Ticket {} #{} Mint Success".format(send_msg.chain, send_msg.token_id)

        # The character encoding for the email.
        CHARSET ="utf-8"
        qrcodes = send_msg.qrcodes
        print("qrcodes = {}".format(qrcodes))
        print("qrcodes len = {}".format(len(qrcodes)))

        # Create a multipart/mixed parent container.
        msg = MIMEMultipart('mixed')
        # Add subject, from and to lines.
        msg['Subject'] = SUBJECT
        msg['From'] = SENDER
        msg['To'] = send_msg.email

        # Create a multipart/alternative child container.
        # msg_body = MIMEMultipart('alternative')
        msg_body = MIMEMultipart('alternative')

        html_content = get_mint_template(send_msg.chain, send_msg.chainid, send_msg.address,
                                         ticket_type=send_msg.ticket_type, token_id=send_msg.token_id).encode("utf-8")
        # print('========html_content===========')
        # print(html_content)
        # print('===================')
        htmlpart = MIMEText(html_content, 'html', CHARSET)

        # Add the text and HTML parts to the child container.
        # msg_body.attach(textpart)
        msg_body.attach(htmlpart)

        if True:
            # for i in range(len(qrcodes)):
            img_data = gen_qrcode_png_bytes(qrcodes[0])
            embed_qrcode = MIMEImage(img_data)
            embed_qrcode.add_header('Content-ID', '<qrcode>')
            embed_qrcode.add_header('Content-Disposition', 'inline', filename="qrcode.png")
            msg_body.attach(embed_qrcode);
        msg.attach(msg_body)

        # Define the attachment part and encode it using MIMEApplication.
        # att = MIMEApplication(open(ATTACHMENT, 'rb').read())
        for i in range(len(qrcodes)):
            att = MIMEApplication(img_data)
            # Add a header to tell the email client to treat this part as an attachment,
            # and to give the attachment a name.
            att.add_header('Content-Disposition','attachment',
                           filename="qrcode_{}_tokenid_{}.png".format(
                               send_msg.chain.replace(' ', '_'), send_msg.token_id))

            # Attach the multipart/alternative child container to the multipart/mixed
            # parent container.
            msg.attach(att)

        # Add the attachment to the parent container.
        # msg.attach(att)
        #print(msg)
        #Provide the contents of the email.
        response = self.client.send_raw_email(
            Source=SENDER,
            Destinations=[

            ],
            RawMessage={
                'Data':msg.as_string(),
            },
            # ConfigurationSetName=CONFIGURATION_SET
        )
        return response