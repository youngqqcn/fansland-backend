

# def get_mint_template(address: str, img_count: int) :
#     first_part = f"""<div style=width:100%; background-color:#E4E5E7;padding:2px;>
#         <div style=max-width:640px; margin:0 auto; background:#ffffff;border-radius:10px; overflow:hidden;>
#             <img src=https://static.fansland.io/email/20240126-114328.jpg style=width:100%  alt='' />
#             <div style=padding:32px;>
#                 <h1 style=font-size:24px; font-weight:bold; text-align:center; color:#081131;> Fansland NFT ticket Mint success! 🎉 </h1>
#                 <p style=margin:0;><br></p>
#                 <p style=font-size:16px; line-height:170%; letter-spacing:0.2px;  margin:0;> Congratulations! 👏</p>
#                 <p style=margin:0;><br></p>
#                 <p style=font-size:16px; line-height:170%; letter-spacing:0.2px; margin:0;> You have successfully mint Fansland NFT tickets with your wallet address: </p>
#                 <p style=margin:0;><br></p>
#                 <p style=font-size:16px; text-align:center;  line-height:170%; letter-spacing:0.2px; margin:0;> {address} </p>
#                 <p style=margin:0;><br></p>
#                 <p style=font-size:16px; line-height:170%; letter-spacing:0.2px; margin:0;>
#                     Please login to <a href="https://fansland.io/">https://fansland.io</a> with this wallet address to see your NFT tickets.
#                 </p>
#                 <p style=margin:0;><br></p>
#                 <hr>
#                 <p style=font-size:14px; font-style: italic;  text-align:center;><em> Fansland aims to build the best Web3 infrastructure for global fans.<em> </p>
#                 """

#     img_part = ""
#     for i in range(img_count):
#         img_part += f"""<img src="cid:qrcode{i}" alt="qrcode{i}">"""
#         img_part += f"""<p style=margin:0;><br></p>"""

#     last_part = """
#             </div>
#         </div>
#     </div>"""

#     return first_part + img_part + last_part


def get_mint_template(address, ticket_type) :
    location = "Thailand Bangkok"
    ticketType = ticket_type
    date = "3-6 May 2024"
    first_part = f"""<div style=width:100%; background-color:#E4E5E7;padding:2px;>
    <div style="max-width:640px; margin:0 auto; background:#ffffff;border-radius:10px; overflow:hidden;">
        <img src=https://static.fansland.io/email/20240126-114328.jpg style=width:100%  alt='' />
        <div style="padding:12px;">
            <h1 style="font-size:18px; font-weight:bold; text-align:center; color:#081131;"> Fansland NFT ticket Mint success! 🎉 </h1>
            <p style="margin:0;"><br></p>
            <p style="font-size:14px; line-height:170%; letter-spacing:0.2px;  margin:0;"> Congratulations! 👏</p>
            <p style="font-size:14px; line-height:170%; letter-spacing:0.2px; margin:0;"> You have successfully mint Fansland NFT ticket with your wallet address: </p>
            <p style="font-size:14px; line-height:170%; font-weight: 600; margin:0;">{address}</p>
            <p style="font-size:14px; line-height:170%; letter-spacing:0.2px; margin:0;">
              Please login to <a href="https://fansland.io/">https://fansland.io</a> with this wallet address to see your NFT tickets.
            </p>
            <p style="margin:0;"><br></p>
            <p style="font-size:16px; line-height:170%; letter-spacing:0.2px; margin:0; font-weight: 600;">
              Ticket Details
            </p>
            <p style="font-size:14px; line-height:170%; letter-spacing:0.2px; margin:0;">
              It is a privilege that you attend our event
            </p>
            <p style="font-size:14px;  line-height:170%; letter-spacing:0.2px; margin:0;">
              We are happy to provide you with the following event details:
            </p>
            <p style="font-size:14px;   line-height:170%; letter-spacing:0.2px; margin:0;">
              Location: <span style="font-weight: 600;">{location}</span>
            </p>
            <p style="font-size:14px;   line-height:170%; letter-spacing:0.2px; margin:0;">
              Ticket Type: <span style="font-weight: 600;">{ticketType}</span>
            </p>
            <p style="font-size:14px;   line-height:170%; letter-spacing:0.2px; margin:0;">
              Organiser: <span style="font-weight: 600;">Fansland(fansland.io)</span>
            </p>
            <p style="font-size:14px;   line-height:170%; letter-spacing:0.2px; margin:0;">
              Date: <span style="font-weight: 600;">{date}</span>
            </p>
            <p style="margin:0;"><br></p>
            <p style="margin:0;"><br></p>
            <p style="margin:0;"><br></p>
            <p style="text-align: center;">
              <img src="cid:qrcode0" alt="qrcode">
            </p>
            <p style="margin:0;"><br></p>
            <p style="margin:0;"><br></p>
            <p style="font-size:14px; text-align:center;  line-height:170%; letter-spacing:0.2px; margin:0;">
              Simply present this QR Code to check-in.
            </p>
            <p style="margin:0;"><br></p>
            <p style="margin:0;"><br></p>
            <p style="text-align: center;">
              <a href="https://fansland.io" target="_blank">
                <button style="border: none; color: #FFF; font-size: 14px; font-weight: 600; border-radius: 12px; padding: 12px 24px; background: linear-gradient(251deg, #AE18FD 8%, #FC6512 90%);">View Event</button>
              </a>
            </p>
            <hr>
            <p style="text-align: center;">
              <img style="width: 200px;" src="https://static.fansland.io/email/20240126-114328.jpg" />
            </p>
            <p style="text-align: center; font-size:16px; font-weight: 500; line-height:170%; letter-spacing:0.2px;">
              Build The Best Web3 Infrastructure for Global FANS
            </p>
            <p style="font-size:13px; line-height:160%; font-style:oblique; letter-spacing:0.2px; margin:0;">
              Fansland Web3 Music Festival is a groundbreaking event that redefines the traditional music festival experience with innovative Web3 technology and culture, making it more immersive, engaging, and inclusive for fans worldwide. See you shortly!
            </p>
            <p style="text-align: center;">
              <a style="margin: 0 6px;" href="https://twitter.com/fansland_io" target="_blank">
                <img style="border-radius: 50%; width: 28px; overflow: hidden;" src="https://miro.medium.com/v2/1*m-R_BkNf1Qjr1YbyOIJY2w.png" alt="">
              </a>
              <a style="margin: 0 ;" href="https://twitter.com/fansland_io" target="_blank">
                <img style="border-radius: 50%; width: 28px; overflow: hidden;"  src="https://abs.twimg.com/favicons/twitter.3.ico" alt="">
              </a>
            </p>
            <hr>
            <p style="color: #807E7E; font-size: 12px; text-align: center;">
              ©2024 Fansland. All rights reserved
            </p>
        </div>
    </div>
</div>"""

    return first_part