"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[5758],{35318:(e,t,n)=>{n.d(t,{Zo:()=>m,kt:()=>u});var o=n(27378);function r(e,t,n){return t in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}function a(e,t){var n=Object.keys(e);if(Object.getOwnPropertySymbols){var o=Object.getOwnPropertySymbols(e);t&&(o=o.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),n.push.apply(n,o)}return n}function i(e){for(var t=1;t<arguments.length;t++){var n=null!=arguments[t]?arguments[t]:{};t%2?a(Object(n),!0).forEach((function(t){r(e,t,n[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(n)):a(Object(n)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(n,t))}))}return e}function l(e,t){if(null==e)return{};var n,o,r=function(e,t){if(null==e)return{};var n,o,r={},a=Object.keys(e);for(o=0;o<a.length;o++)n=a[o],t.indexOf(n)>=0||(r[n]=e[n]);return r}(e,t);if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);for(o=0;o<a.length;o++)n=a[o],t.indexOf(n)>=0||Object.prototype.propertyIsEnumerable.call(e,n)&&(r[n]=e[n])}return r}var s=o.createContext({}),p=function(e){var t=o.useContext(s),n=t;return e&&(n="function"==typeof e?e(t):i(i({},t),e)),n},m=function(e){var t=p(e.components);return o.createElement(s.Provider,{value:t},e.children)},c={inlineCode:"code",wrapper:function(e){var t=e.children;return o.createElement(o.Fragment,{},t)}},g=o.forwardRef((function(e,t){var n=e.components,r=e.mdxType,a=e.originalType,s=e.parentName,m=l(e,["components","mdxType","originalType","parentName"]),g=p(n),u=r,h=g["".concat(s,".").concat(u)]||g[u]||c[u]||a;return n?o.createElement(h,i(i({ref:t},m),{},{components:n})):o.createElement(h,i({ref:t},m))}));function u(e,t){var n=arguments,r=t&&t.mdxType;if("string"==typeof e||r){var a=n.length,i=new Array(a);i[0]=g;var l={};for(var s in t)hasOwnProperty.call(t,s)&&(l[s]=t[s]);l.originalType=e,l.mdxType="string"==typeof e?e:r,i[1]=l;for(var p=2;p<a;p++)i[p]=n[p];return o.createElement.apply(null,i)}return o.createElement.apply(null,n)}g.displayName="MDXCreateElement"},23426:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>s,contentTitle:()=>i,default:()=>c,frontMatter:()=>a,metadata:()=>l,toc:()=>p});var o=n(25773),r=(n(27378),n(35318));const a={slug:"proto-v0.11",title:"proto v0.11 - New shims and better logging",authors:["milesj"],tags:["proto","shim","logging"]},i=void 0,l={permalink:"/blog/proto-v0.11",editUrl:"https://github.com/moonrepo/moon/tree/master/website/blog/2023-06-25_proto-v0.11.mdx",source:"@site/blog/2023-06-25_proto-v0.11.mdx",title:"proto v0.11 - New shims and better logging",description:"This is a small release that improves shims and logs.",date:"2023-06-25T00:00:00.000Z",formattedDate:"June 25, 2023",tags:[{label:"proto",permalink:"/blog/tags/proto"},{label:"shim",permalink:"/blog/tags/shim"},{label:"logging",permalink:"/blog/tags/logging"}],readingTime:1.04,hasTruncateMarker:!0,authors:[{name:"Miles Johnson",title:"Founder, developer",url:"https://github.com/milesj",imageURL:"/img/authors/miles.jpg",key:"milesj"}],frontMatter:{slug:"proto-v0.11",title:"proto v0.11 - New shims and better logging",authors:["milesj"],tags:["proto","shim","logging"]},prevItem:{title:"moon v1.9 - VCS hooks management and improved task inheritance",permalink:"/blog/moon-v1.9"},nextItem:{title:"moon v1.8 - Code owners and shared configuration",permalink:"/blog/moon-v1.8"}},s={authorsImageUrls:[void 0]},p=[{value:"New and improved shims",id:"new-and-improved-shims",level:2},{value:"Better logging",id:"better-logging",level:2},{value:"Other changes",id:"other-changes",level:2}],m={toc:p};function c(e){let{components:t,...n}=e;return(0,r.kt)("wrapper",(0,o.Z)({},m,n,{components:t,mdxType:"MDXLayout"}),(0,r.kt)("p",null,"This is a small release that improves shims and logs."),(0,r.kt)("h2",{id:"new-and-improved-shims"},"New and improved shims"),(0,r.kt)("p",null,"The core facet of proto is our shims found at ",(0,r.kt)("inlineCode",{parentName:"p"},"~/.proto/bin"),". They exist purely to re-route tool\nexecutions internally to proto, so that we can detect the correct version of these tools to run.\nHowever, maintaining and creating these shims has historically been very complicated. So we chose to\nrewrite them from the ground-up!"),(0,r.kt)("p",null,'All tools should continue to function exactly as they did before, if not better. Furthermore,\nbecause of this new shim layer, we\'re now able to create what we call "secondary shims", like\n',(0,r.kt)("a",{parentName:"p",href:"https://bun.sh/docs/cli/bunx"},(0,r.kt)("inlineCode",{parentName:"a"},"bunx")," for Bun"),", ",(0,r.kt)("inlineCode",{parentName:"p"},"pnpx")," for pnpm, and ",(0,r.kt)("inlineCode",{parentName:"p"},"yarnpkg")," for Yarn."),(0,r.kt)("h2",{id:"better-logging"},"Better logging"),(0,r.kt)("p",null,"proto has supported logging since its initial release behind the ",(0,r.kt)("inlineCode",{parentName:"p"},"PROTO_LOG")," environment variable.\nHowever, this variable wasn't heavily documented, nor easily discoverable. So as an alternative, we\nnow support a global ",(0,r.kt)("inlineCode",{parentName:"p"},"--log")," option, which can be passed to any ",(0,r.kt)("inlineCode",{parentName:"p"},"proto")," command."),(0,r.kt)("pre",null,(0,r.kt)("code",{parentName:"pre",className:"language-shell"},"$ proto install node --log trace\n")),(0,r.kt)("p",null,"On top of this, we also ran an audit of all our log calls, to improve messaging, include additional\ninformation, rework applicable levels, and more. They should be far more readable!"),(0,r.kt)("h2",{id:"other-changes"},"Other changes"),(0,r.kt)("p",null,"View the ",(0,r.kt)("a",{parentName:"p",href:"https://github.com/moonrepo/proto/releases/tag/v0.11.0"},"official release")," for a full list\nof changes."))}c.isMDXComponent=!0}}]);