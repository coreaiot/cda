import { Server } from 'socket.io';

const io = new Server(44444, {
  cors: {
    origin: '*',
  },
});

io.on('connect', async socket => {
  console.log(socket.id, 'connected');

  socket.on('test', e => {
    console.log('test', e);
  });
  while(true) {
    socket.emit('test', 'test');
    socket.emit('test', Buffer.alloc(4));
    await new Promise(r => setTimeout(r, 1000));
  }
});