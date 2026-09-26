/**
 * ChatPage — Room list sidebar, chat window, and create room functionality.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';
import { ChatWindow } from '../components/chat-window.js';

export class ChatPage {
  constructor(container) {
    this.container = container;
    this.rooms = [];
    this.activeRoom = null;
    this.chatWindow = null;
    this._render();
    this._loadRooms();
  }

  /**
   * Render the chat page layout.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'chat-page';

    this.layout = createElement('div', { class: 'chat-layout' });

    // Sidebar with room list
    this.sidebar = createElement('div', { class: 'chat-sidebar' });
    const sidebarHeader = createElement('div', { class: 'map-page-sidebar-header' });
    sidebarHeader.appendChild(createElement('h2', { class: 'section-title' }, 'Chats'));

    const createBtn = createElement('button', { class: 'btn btn-sm btn-primary', style: { marginTop: '8px' } }, '+ New Room');
    createBtn.addEventListener('click', () => this._createRoom());
    sidebarHeader.appendChild(createBtn);

    this.sidebar.appendChild(sidebarHeader);

    this.roomList = createElement('div', { class: 'room-list' });
    this.sidebar.appendChild(this.roomList);

    this.layout.appendChild(this.sidebar);

    // Chat window container
    this.chatContainer = createElement('div', { class: 'chat-main' });
    this.layout.appendChild(this.chatContainer);

    this.container.appendChild(this.layout);
  }

  /**
   * Load chat rooms.
   * @private
   */
  async _loadRooms() {
    try {
      this.rooms = await api.get('/chat/rooms');
      this._renderRoomList();
    } catch (err) {
      console.error('[ChatPage] Failed to load rooms:', err);
    }
  }

  /**
   * Render the room list.
   * @private
   */
  _renderRoomList() {
    clearElement(this.roomList);

    for (const room of this.rooms) {
      const item = createElement('div', {
        class: `room-item ${this.activeRoom?.id === room.id ? 'active' : ''}`,
        'data-room-id': room.id,
      });

      const avatar = createElement('div', { class: 'avatar avatar-sm' });
      avatar.style.background = '#6366f1';
      avatar.textContent = room.name?.charAt(0) || '?';

      const name = createElement('div', { class: 'room-item-name' }, room.name);

      item.appendChild(avatar);
      item.appendChild(name);

      if (room.unreadCount > 0) {
        item.appendChild(createElement('span', { class: 'room-item-badge' }, String(room.unreadCount)));
      }

      item.addEventListener('click', () => this._selectRoom(room));
      this.roomList.appendChild(item);
    }
  }

  /**
   * Select a room and open its chat.
   * @private
   */
  _selectRoom(room) {
    this.activeRoom = room;
    this._renderRoomList();

    if (this.chatWindow) {
      this.chatWindow.destroy();
    }

    this.chatWindow = new ChatWindow(this.chatContainer, room.id);

    // Load message history
    api.get(`/chat/rooms/${room.id}/messages`)
      .then(messages => {
        for (const msg of messages) {
          this.chatWindow.addMessage(msg);
        }
      })
      .catch(err => console.error('[ChatPage] Failed to load messages:', err));
  }

  /**
   * Create a new chat room.
   * @private
   */
  async _createRoom() {
    const name = prompt('Room name:');
    if (!name?.trim()) return;

    try {
      const room = await api.post('/chat/rooms', { name: name.trim() });
      this.rooms.push(room);
      this._renderRoomList();
      this._selectRoom(room);
    } catch (err) {
      console.error('[ChatPage] Failed to create room:', err);
    }
  }

  /**
   * Destroy the page.
   */
  destroy() {
    this.chatWindow?.destroy();
    clearElement(this.container);
  }
}

export default ChatPage;
