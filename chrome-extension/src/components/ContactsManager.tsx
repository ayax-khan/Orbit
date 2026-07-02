import React, { useCallback, useEffect, useState } from 'react';
import { useAuthStore } from '../stores/authStore';

interface Contact {
  id: string;
  user_id: string;
  email: string;
  full_name: string;
  display_name: string;
  created_at: string;
}

interface UserSearchResult {
  id: string;
  email: string;
  full_name: string;
}

interface ContactsManagerProps {
  onConnect?: (contactUserId: string) => void;
}

export const ContactsManager: React.FC<ContactsManagerProps> = ({ onConnect }) => {
  const token = useAuthStore((state) => state.token);
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [searchResults, setSearchResults] = useState<UserSearchResult[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showSearch, setShowSearch] = useState(false);

  const loadContacts = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    setError(null);
    try {
      const response = await fetch('http://localhost:8080/api/v1/contacts', {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!response.ok) throw new Error('Failed to load contacts');
      const data = await response.json();
      setContacts(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load contacts');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => {
    loadContacts();
  }, [loadContacts]);

  const handleSearch = async (query: string) => {
    setSearchQuery(query);
    if (!query || !token) {
      setSearchResults([]);
      return;
    }
    try {
      const response = await fetch(
        `http://localhost:8080/api/v1/contacts/search?q=${encodeURIComponent(query)}`,
        {
          headers: { Authorization: `Bearer ${token}` },
        }
      );
      if (!response.ok) throw new Error('Search failed');
      const data = await response.json();
      setSearchResults(data);
    } catch (err) {
      console.error('Search error:', err);
    }
  };

  const handleAddContact = async (user: UserSearchResult) => {
    if (!token) return;
    try {
      const response = await fetch('http://localhost:8080/api/v1/contacts/add', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({
          contact_user_id: user.id,
          display_name: user.full_name,
        }),
      });
      if (!response.ok) throw new Error('Failed to add contact');
      await loadContacts();
      setSearchQuery('');
      setSearchResults([]);
      setShowSearch(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add contact');
    }
  };

  const handleRemoveContact = async (contactId: string) => {
    if (!token) return;
    try {
      const response = await fetch(
        `http://localhost:8080/api/v1/contacts/${contactId}`,
        {
          method: 'DELETE',
          headers: { Authorization: `Bearer ${token}` },
        }
      );
      if (!response.ok) throw new Error('Failed to remove contact');
      await loadContacts();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to remove contact');
    }
  };

  return (
    <div className="w-full p-4">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-xl font-bold">Contacts</h2>
        <button
          onClick={() => setShowSearch(!showSearch)}
          className="text-sm px-3 py-1 bg-blue-600 text-white rounded"
        >
          {showSearch ? 'Cancel' : 'Add Contact'}
        </button>
      </div>

      {error && (
        <div className="mb-3 p-2 text-sm text-red-700 bg-red-100 rounded">{error}</div>
      )}

      {showSearch && (
        <div className="mb-4 p-3 border rounded bg-gray-50">
          <input
            type="text"
            placeholder="Search by email or name..."
            value={searchQuery}
            onChange={(e) => handleSearch(e.target.value)}
            className="w-full p-2 border rounded mb-2"
          />
          {searchResults.length > 0 && (
            <div className="max-h-40 overflow-y-auto">
              {searchResults.map((user) => (
                <div
                  key={user.id}
                  className="flex justify-between items-center p-2 border-b last:border-b-0"
                >
                  <div>
                    <div className="font-medium">{user.full_name}</div>
                    <div className="text-sm text-gray-600">{user.email}</div>
                  </div>
                  <button
                    onClick={() => handleAddContact(user)}
                    className="text-sm px-2 py-1 bg-green-600 text-white rounded"
                  >
                    Add
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {loading ? (
        <p className="text-gray-500">Loading contacts…</p>
      ) : contacts.length === 0 ? (
        <div className="text-gray-500">
          <p>No contacts yet.</p>
          <p className="text-sm mt-1">Add contacts to start screen sharing with friends.</p>
        </div>
      ) : (
        <div className="space-y-2">
          {contacts.map((contact) => (
            <div
              key={contact.id}
              className="flex justify-between items-center p-2 border rounded"
            >
              <div>
                <div className="font-medium">{contact.display_name}</div>
                <div className="text-sm text-gray-600">{contact.email}</div>
              </div>
              <div className="flex gap-2">
                {onConnect && (
                  <button
                    onClick={() => onConnect(contact.user_id)}
                    className="text-sm px-2 py-1 bg-green-600 text-white rounded"
                  >
                    Connect
                  </button>
                )}
                <button
                  onClick={() => handleRemoveContact(contact.id)}
                  className="text-sm px-2 py-1 bg-red-600 text-white rounded"
                >
                  Remove
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
