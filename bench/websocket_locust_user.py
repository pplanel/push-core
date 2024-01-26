import gevent
from locust import User
import websocket
import logging
from locust.events import request_success


class WebsocketUser(User):
    """
    A websockets User
    """

    abstract = True

    def connect(self, host: str, header=[], **kwargs):
        self.ws = websocket.create_connection(host, header=header, **kwargs)
        self.ws_greenlet = gevent.spawn(self.receive_loop)

    def on_message(self, message):
        logging.debug(f"WSS: {message}")
        self.environment.events.request.fire(
            request_type="WSR",
            name="asda",
            response_time=None,
            response_length=len(message),
            exception=None,
            context=self.context(),
        )

    def receive_loop(self):
        while True:
            message = self.ws.recv()
            logging.debug(f"WSR: {message}")
            self.on_message(message)

    def send(self, body, name=None, context={}, opcode=websocket.ABNF.OPCODE_TEXT):
        self.environment.events.request.fire(
            request_type="WSS",
            name=name,
            response_time=None,
            response_length=len(body),
            exception=None,
            context={**self.context(), **context},
        )
        logging.debug(f"WSS: {body}")
        self.ws.send(body, opcode)

    def sleep_with_heartbeat(self, seconds):
        while seconds >= 0:
            gevent.sleep(min(15, seconds))
            seconds -= 15
            self.send("2")
