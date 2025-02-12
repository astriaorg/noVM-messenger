import { useState, ChangeEvent, KeyboardEvent, useEffect, useRef } from 'react';
import './App.css';
import './global.css';

// Define a type for the message object
type Message = {
  text: string;
  sender: string;
  from: 'left' | 'right';
};

// bestest random ever
const senderId = Math.random().toString(36).substr(2);

function App() {
  // Use the Message type for the messages state
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputValue, setInputValue] = useState<string>('');
  const ws = useRef<WebSocket | null>(null);

  // Set latest message ref
  const endOfMessagesRef = useRef<null | HTMLDivElement>(null);

  const sender = `user-${senderId}`;

  const avatars = [
    'nes-mario',
    'nes-ash',
    'nes-pokeball',
    'nes-bulbasaur',
    'nes-charmander',
    'nes-squirtle',
    'nes-kirby',
  ];

  const getAvatar = (id: string) => {
    const hashId = id.split('').reduce((a, b) => ((a << 5) - a) + b.charCodeAt(0), 0);
    return avatars[Math.abs(hashId) % avatars.length];
  };

  // load recent messages
  useEffect(() => {
    console.debug('Fetching recent messages at:', import.meta.env.VITE_APP_API_URL);
    fetch(`${import.meta.env.VITE_APP_API_URL}/recent`)
      .then(response => response.json())
      .then(data => {
        console.debug('Recent messages:', data);
        console.log(data);
        const newMessages: Message[] = [];
        for (const msg of data) {
          console.log(msg);
          const message: Message = {
            text: msg.message,
            sender: msg.sender,
            from: msg.sender !== sender ? 'left' : 'right',
          };
          newMessages.push(message);
        }
        setMessages(newMessages);
      })
      .catch(error => console.error('Error:', error));
  }, []);

  // get messages from rollup ws
  useEffect(() => {
    ws.current = new WebSocket(import.meta.env.VITE_APP_WEBSOCKET_URL);
    ws.current.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data && data.length) {
        const newMessages: Message[] = [];
        for (const msg of data) {
          if (msg.message && msg.sender !== sender) {
            const message: Message = {
              text: msg.message,
              sender: msg.sender,
              from: 'left',
            };
            newMessages.push(message);
          }
        }
        setMessages((prevMessages) => [...prevMessages, ...newMessages]);
      }
    };
    return () => {
      ws.current?.close();
    };
  }, [sender]);

  // Snap to latest message
  useEffect(() => {
    endOfMessagesRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSendMessage = () => {
    if (inputValue.trim()) {
      // Send to rollup api
      console.debug('Sending message:', inputValue);
      fetch(`${import.meta.env.VITE_APP_API_URL}/message`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          sender,
          message: inputValue
        })
      });
      // be optimistic. Append a new message to the messages array
      setMessages([...messages, { text: inputValue, sender, from: 'right' }]);
      setInputValue('');
    }
  };

  // Add the type for the event parameter
  const handleInputChange = (event: ChangeEvent<HTMLInputElement>) => {
    setInputValue(event.target.value);
  };

  // Add the type for the event parameter
  const handleKeyPress = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      handleSendMessage();
    }
  };

  return (
    <>
      <div className="nes-container with-title is-centered">
        <div className="title">
          <p>Modular Chat</p>
        </div>
        <div className="message-list">
          {messages.map((message, index) => (
            <section key={index} className={`message -${message.from}`}>
              <div className={`nes-balloon from-${message.from}`}>
                <p>{message.sender}: {message.text}</p>
              </div>
            </section>
          ))}
          <div ref={endOfMessagesRef} />
        </div>
        <div className="nes-field is-inline">
          <input
            type="text"
            className="nes-input"
            placeholder="Type a message..."
            value={inputValue}
            onChange={handleInputChange}
            onKeyPress={handleKeyPress}
          />
          <button
            type="button"
            className="nes-btn is-primary"
            onClick={handleSendMessage}
          >
            Send
          </button>
        </div>
      </div>
      <div className="footer">
        <p>built by <a href="https://astria.org" target="_blank">Astria</a> with <a href="https://celestia.org/" target="_blank">Celestia</a> underneath</p>
        <p>
          <a href="https://twitter.com/AstriaOrg" target="_blank">
            <i className="nes-icon close is-medium"></i>
          </a>
          <a href="https://github.com/astriaorg/messenger-rollup" target="_blank">
            <i className="nes-icon github is-medium"></i>
          </a>
        </p>
      </div>
    </>
  );
}

export default App;
