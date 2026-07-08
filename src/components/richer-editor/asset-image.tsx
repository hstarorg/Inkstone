import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import Image, { type ImageOptions } from "@tiptap/extension-image";
import { ReactNodeViewRenderer } from "@tiptap/react";
import { ImageView } from "./image-view";

export type ResolveAssetSrc = (src: string) => string;
export type UploadImage = (file: File) => Promise<string>;

export function createAssetImage(resolveSrc: ResolveAssetSrc) {
  return Image.extend({
    addOptions() {
      return { ...(this.parent?.() ?? ({} as ImageOptions)), resolveSrc };
    },
    addAttributes() {
      return {
        ...this.parent?.(),
        width: {
          default: null,
          parseHTML: (element) => {
            const value = element.getAttribute("width");
            return value ? Number.parseInt(value, 10) : null;
          },
          renderHTML: (attributes) =>
            attributes.width ? { width: attributes.width } : {},
        },
      };
    },
    addNodeView() {
      return ReactNodeViewRenderer(ImageView);
    },
  });
}

export function createImageDropPaste(upload: UploadImage | undefined) {
  return Extension.create({
    name: "imageDropPaste",

    addProseMirrorPlugins() {
      if (!upload) return [];
      const editor = this.editor;

      const insertImages = (files: FileList | null | undefined): boolean => {
        const images = Array.from(files ?? []).filter((file) =>
          file.type.startsWith("image/"),
        );
        if (images.length === 0) return false;
        for (const file of images) {
          void upload(file).then((src) => {
            editor.chain().focus().setImage({ src }).run();
          });
        }
        return true;
      };

      return [
        new Plugin({
          key: new PluginKey("imageDropPaste"),
          props: {
            handlePaste: (_view, event) =>
              insertImages(event.clipboardData?.files),
            handleDrop: (_view, event) =>
              insertImages(event.dataTransfer?.files),
          },
        }),
      ];
    },
  });
}
