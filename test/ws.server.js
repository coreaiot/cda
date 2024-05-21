import { WebSocketServer } from 'ws';

const wss = new WebSocketServer({ port: 44444 });

wss.on('connection', function connection(ws) {
  ws.on('error', console.error);

  ws.on('message', function message(data) {
    console.log('received', data);
    ws.send('something');
    ws.send(Buffer.from('something'));
  });
});
