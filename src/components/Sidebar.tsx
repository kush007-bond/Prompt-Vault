import { useState } from 'react';
import { useAppStore } from '../store';
import { Button } from './ui/button';
import { Input } from './ui/input';
import {
  FolderPlus,
  Plus,
  Search,
  Folder,
  Tag,
  Trash2,
  X,
  PlusCircle,
  Moon,
  Sun,
  Monitor,
  Settings,
  Upload,
  Download,
} from 'lucide-react';
import { cn } from '../lib/utils';

interface SidebarProps {
  onOpenSettings: () => void;
  onExport: () => void;
  onImport: () => void;
  width?: number;
}

export function Sidebar({ onOpenSettings, onExport, onImport, width = 256 }: SidebarProps) {
  const {
    collections,
    tags,
    selectedCollectionId,
    setSelectedCollection,
    searchQuery,
    setSearchQuery,
    createCollection,
    createTag,
    deleteCollection,
    deleteTag,
    theme,
    setTheme,
  } = useAppStore();

  const [isAddingCollection, setIsAddingCollection] = useState(false);
  const [isAddingTag, setIsAddingTag] = useState(false);
  const [newCollectionName, setNewCollectionName] = useState('');
  const [newTagName, setNewTagName] = useState('');

  const handleCreateCollection = async () => {
    if (newCollectionName.trim()) {
      await createCollection(newCollectionName.trim());
      setNewCollectionName('');
      setIsAddingCollection(false);
    }
  };

  const handleCreateTag = async () => {
    if (newTagName.trim()) {
      await createTag(newTagName.trim());
      setNewTagName('');
      setIsAddingTag(false);
    }
  };

  return (
    <div className="h-full border-r bg-card flex flex-col overflow-hidden" style={{ width, minWidth: width, maxWidth: width }}>
      {/* Search */}
      <div className="p-3 border-b">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search prompts..."
            className="pl-9"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
      </div>

      {/* Collections */}
      <div className="flex-1 overflow-y-auto p-3">
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-sm font-medium text-muted-foreground">Collections</h3>
          <Button 
            variant="ghost" 
            size="icon" 
            className="h-6 w-6"
            onClick={() => setIsAddingCollection(true)}
          >
            <FolderPlus className="h-4 w-4" />
          </Button>
        </div>

        {/* Add collection input */}
        {isAddingCollection && (
          <div className="flex gap-1 mb-2">
            <Input
              placeholder="Collection name"
              value={newCollectionName}
              onChange={(e) => setNewCollectionName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleCreateCollection();
                if (e.key === 'Escape') setIsAddingCollection(false);
              }}
              autoFocus
              className="h-7 text-sm"
            />
            <Button size="sm" onClick={handleCreateCollection} className="h-7">
              <Plus className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="sm" onClick={() => setIsAddingCollection(false)} className="h-7">
              <X className="h-4 w-4" />
            </Button>
          </div>
        )}
        
        <div className="space-y-1">
          <button
            onClick={() => setSelectedCollection(null)}
            className={cn(
              "w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors",
              selectedCollectionId === null
                ? "bg-primary/10 text-primary"
                : "hover:bg-muted"
            )}
          >
            <Folder className="h-4 w-4" />
            All Prompts
          </button>
          
          {collections.map((collection) => (
            <div
              key={collection.id}
              className={cn(
                "w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors group",
                selectedCollectionId === collection.id
                  ? "bg-primary/10 text-primary"
                  : "hover:bg-muted"
              )}
            >
              <button
                className="flex-1 flex items-center gap-2 text-left"
                onClick={() => setSelectedCollection(collection.id)}
              >
                <Folder className="h-4 w-4" style={{ color: collection.color || undefined }} />
                {collection.name}
              </button>
              <button
                onClick={() => deleteCollection(collection.id)}
                className="opacity-0 group-hover:opacity-100 h-6 w-6 flex items-center justify-center rounded hover:bg-destructive/10 text-destructive"
              >
                <Trash2 className="h-3 w-3" />
              </button>
            </div>
          ))}
        </div>

        {/* Tags */}
        <div className="mt-6">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-sm font-medium text-muted-foreground">Tags</h3>
            <Button 
              variant="ghost" 
              size="icon" 
              className="h-6 w-6"
              onClick={() => setIsAddingTag(true)}
            >
              <PlusCircle className="h-4 w-4" />
            </Button>
          </div>

          {/* Add tag input */}
          {isAddingTag && (
            <div className="flex gap-1 mb-2">
              <Input
                placeholder="Tag name"
                value={newTagName}
                onChange={(e) => setNewTagName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleCreateTag();
                  if (e.key === 'Escape') setIsAddingTag(false);
                }}
                autoFocus
                className="h-7 text-sm"
              />
              <Button size="sm" onClick={handleCreateTag} className="h-7">
                <Plus className="h-4 w-4" />
              </Button>
              <Button variant="ghost" size="sm" onClick={() => setIsAddingTag(false)} className="h-7">
                <X className="h-4 w-4" />
              </Button>
            </div>
          )}
          
          <div className="flex flex-wrap gap-1">
            {tags.map((tag) => (
              <span
                key={tag.id}
                className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs bg-muted hover:bg-muted/80 cursor-pointer"
              >
                <Tag className="h-3 w-3" style={{ color: tag.color || undefined }} />
                {tag.name}
                <button
                  onClick={() => deleteTag(tag.id)}
                  className="hover:text-destructive"
                >
                  ×
                </button>
              </span>
            ))}
          </div>
        </div>
      </div>

      {/* Bottom actions */}
      <div className="p-3 border-t space-y-1">
        <div className="flex gap-1">
          <Button
            variant="ghost"
            size="sm"
            className="flex-1 justify-start gap-2 text-xs"
            onClick={onImport}
            title="Import prompts from JSON"
          >
            <Upload className="h-3.5 w-3.5" />
            Import
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="flex-1 justify-start gap-2 text-xs"
            onClick={onExport}
            title="Export prompts to JSON"
          >
            <Download className="h-3.5 w-3.5" />
            Export
          </Button>
        </div>
        <Button
          variant="ghost"
          className="w-full justify-start gap-2"
          onClick={() => setTheme(theme === 'dark' ? 'light' : theme === 'light' ? 'system' : 'dark')}
        >
          {theme === 'dark' ? <Moon className="h-4 w-4" /> : theme === 'light' ? <Sun className="h-4 w-4" /> : <Monitor className="h-4 w-4" />}
          {theme === 'dark' ? 'Dark' : theme === 'light' ? 'Light' : 'System'}
        </Button>
        <Button
          variant="ghost"
          className="w-full justify-start gap-2"
          onClick={onOpenSettings}
        >
          <Settings className="h-4 w-4" />
          AI Settings
        </Button>
      </div>
    </div>
  );
}