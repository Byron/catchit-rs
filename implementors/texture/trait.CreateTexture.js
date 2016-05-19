(function() {var implementors = {};
implementors['opengl_graphics'] = ["impl <a class='trait' href='texture/trait.CreateTexture.html' title='texture::CreateTexture'>CreateTexture</a>&lt;<a class='primitive' href='https://doc.rust-lang.org/nightly/std/primitive.tuple.html'>()</a>&gt; for <a class='struct' href='opengl_graphics/struct.Texture.html' title='opengl_graphics::Texture'>Texture</a>",];implementors['gfx_texture'] = ["impl&lt;F, R&gt; <a class='trait' href='texture/trait.CreateTexture.html' title='texture::CreateTexture'>CreateTexture</a>&lt;F&gt; for <a class='struct' href='gfx_texture/struct.Texture.html' title='gfx_texture::Texture'>Texture</a>&lt;R&gt; <span class='where'>where F: <a class='trait' href='gfx_core/factory/trait.Factory.html' title='gfx_core::factory::Factory'>Factory</a>&lt;R&gt;, R: <a class='trait' href='gfx_core/trait.Resources.html' title='gfx_core::Resources'>Resources</a></span>",];implementors['piston_window'] = ["impl&lt;F, R&gt; <a class='trait' href='texture/trait.CreateTexture.html' title='texture::CreateTexture'>CreateTexture</a>&lt;F&gt; for <a class='struct' href='piston_window/struct.Texture.html' title='piston_window::Texture'>Texture</a>&lt;R&gt; <span class='where'>where R: <a class='trait' href='gfx_core/trait.Resources.html' title='gfx_core::Resources'>Resources</a>, F: <a class='trait' href='gfx_core/factory/trait.Factory.html' title='gfx_core::factory::Factory'>Factory</a>&lt;R&gt;</span>",];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
